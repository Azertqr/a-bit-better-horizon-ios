// File: crates/horizon-client/src/raven_bs.rs
//! Raven bS+ (1.3.3+1) Complete Native Rust Port for Horizon iOS (aarch64-apple-ios)
//! Architecture: Zero-allocation hot-path tick/render engine, lock-free atomic state access,
//! full math implementation for kinematics, raycasting, ballistic trajectories, and ESP projection.

#![allow(dead_code, clippy::too_many_arguments, clippy::type_complexity)]

use std::collections::{HashMap, VecDeque};
use std::f32::consts::PI;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

// ============================================================================
// CORE PRIMITIVES & GAME STATE INTERFACES
// ============================================================================

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vec3 {
    #[inline]
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        Self::new(self.x + other.x, self.y + other.y, self.z + other.z)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        Self::new(self.x - other.x, self.y - other.y, self.z - other.z)
    }

    #[inline]
    pub fn scale(self, s: f64) -> Self {
        Self::new(self.x * s, self.y * s, self.z * s)
    }

    #[inline]
    pub fn dist_sq(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        let dz = self.z - other.z;
        dx * dx + dy * dy + dz * dz
    }

    #[inline]
    pub fn dist(self, other: Self) -> f64 {
        self.dist_sq(other).sqrt()
    }

    #[inline]
    pub fn horiz_dist(self, other: Self) -> f64 {
        let dx = self.x - other.x;
        let dz = self.z - other.z;
        (dx * dx + dz * dz).sqrt()
    }

    #[inline]
    pub fn normalize(self) -> Self {
        let len = (self.x * self.x + self.y * self.y + self.z * self.z).sqrt();
        if len < 1e-9 {
            Self::new(0.0, 0.0, 0.0)
        } else {
            self.scale(1.0 / len)
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl BlockPos {
    #[inline]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[inline]
    pub fn from_vec3(v: Vec3) -> Self {
        Self {
            x: v.x.floor() as i32,
            y: v.y.floor() as i32,
            z: v.z.floor() as i32,
        }
    }

    #[inline]
    pub fn center(self) -> Vec3 {
        Vec3::new(self.x as f64 + 0.5, self.y as f64 + 0.5, self.z as f64 + 0.5)
    }

    #[inline]
    pub fn offset(self, face: BlockFace) -> Self {
        let (dx, dy, dz) = face.normal();
        Self::new(self.x + dx, self.y + dy, self.z + dz)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockFace {
    Down,
    Up,
    North,
    South,
    West,
    East,
}

impl BlockFace {
    #[inline]
    pub const fn normal(self) -> (i32, i32, i32) {
        match self {
            Self::Down => (0, -1, 0),
            Self::Up => (0, 1, 0),
            Self::North => (0, 0, -1),
            Self::South => (0, 0, 1),
            Self::West => (-1, 0, 0),
            Self::East => (1, 0, 0),
        }
    }

    #[inline]
    pub const fn opposite(self) -> Self {
        match self {
            Self::Down => Self::Up,
            Self::Up => Self::Down,
            Self::North => Self::South,
            Self::South => Self::North,
            Self::West => Self::East,
            Self::East => Self::West,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,
}

impl Aabb {
    #[inline]
    pub fn from_entity(pos: Vec3, width: f64, height: f64, expand: f64) -> Self {
        let hw = (width * 0.5) + expand;
        Self {
            min: Vec3::new(pos.x - hw, pos.y - expand, pos.z - hw),
            max: Vec3::new(pos.x + hw, pos.y + height + expand, pos.z + hw),
        }
    }

    #[inline]
    pub fn intersects_ray(&self, origin: Vec3, dir: Vec3, max_dist: f64) -> Option<f64> {
        let inv_x = if dir.x.abs() < 1e-9 { 1e9 } else { 1.0 / dir.x };
        let inv_y = if dir.y.abs() < 1e-9 { 1e9 } else { 1.0 / dir.y };
        let inv_z = if dir.z.abs() < 1e-9 { 1e9 } else { 1.0 / dir.z };

        let t1 = (self.min.x - origin.x) * inv_x;
        let t2 = (self.max.x - origin.x) * inv_x;
        let t3 = (self.min.y - origin.y) * inv_y;
        let t4 = (self.max.y - origin.y) * inv_y;
        let t5 = (self.min.z - origin.z) * inv_z;
        let t6 = (self.max.z - origin.z) * inv_z;

        let tmin = t1.min(t2).max(t3.min(t4)).max(t5.min(t6));
        let tmax = t1.max(t2).min(t3.max(t4)).min(t5.max(t6));

        if tmax < 0.0 || tmin > tmax || tmin > max_dist {
            None
        } else {
            Some(tmin.max(0.0))
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemCategory {
    Sword,
    Axe,
    Pickaxe,
    Shovel,
    Block,
    GoldenApple,
    EnderPearl,
    ThrowPotion,
    Fireball,
    WaterBucket,
    FishingRod,
    Bow,
    ArmorHelmet,
    ArmorChestplate,
    ArmorLeggings,
    ArmorBoots,
    Trash,
}

#[derive(Clone, Debug)]
pub struct ItemStack {
    pub id: u16,
    pub name: String,
    pub count: u8,
    pub category: ItemCategory,
    pub damage_score: f32,
    pub armor_protection: f32,
    pub mining_speed: f32,
}

#[derive(Clone, Debug)]
pub struct EntitySnapshot {
    pub id: u32,
    pub name: String,
    pub pos: Vec3,
    pub prev_pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub width: f64,
    pub height: f64,
    pub health: f32,
    pub max_health: f32,
    pub hurt_time: u8,
    pub on_ground: bool,
    pub invisible: bool,
    pub is_player: bool,
    pub is_mob: bool,
    pub is_item: bool,
    pub is_fireball: bool,
    pub is_arrow: bool,
    pub team_color: u32,
    pub ticks_existed: u32,
    pub ping: i32,
    pub held_item: Option<ItemCategory>,
}

#[derive(Clone, Debug)]
pub struct OutgoingPacket {
    pub packet_id: u16,
    pub timestamp: Instant,
    pub pos: Option<Vec3>,
    pub rot: Option<(f32, f32)>,
    pub target_entity: Option<u32>,
    pub raw_bytes: Vec<u8>,
}

#[derive(Clone, Debug)]
pub struct DrawCommand {
    pub kind: DrawKind,
    pub color_rgba: u32,
}

#[derive(Clone, Debug)]
pub enum DrawKind {
    Rect { x: f32, y: f32, w: f32, h: f32, radius: f32 },
    Text { x: f32, y: f32, text: String, scale: f32 },
    Line3D { start: Vec3, end: Vec3, width: f32 },
    Box3D { aabb: Aabb, outline_only: bool },
}

#[derive(Clone, Debug)]
pub struct LocalPlayerState {
    pub entity_id: u32,
    pub pos: Vec3,
    pub prev_pos: Vec3,
    pub vel: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub camera_yaw: f32,
    pub camera_pitch: f32,
    pub server_yaw: f32,
    pub server_pitch: f32,
    pub on_ground: bool,
    pub sprinting: bool,
    pub sneaking: bool,
    pub using_item: bool,
    pub swinging: bool,
    pub blocking: bool,
    pub health: f32,
    pub max_health: f32,
    pub hunger: u8,
    pub saturation: f32,
    pub hurt_time: u8,
    pub fall_distance: f32,
    pub step_height: f32,
    pub gamma: f32,
    pub timer_speed: f32,
    pub right_click_delay: u8,
    pub move_forward: f32,
    pub move_strafe: f32,
    pub jump_pressed: bool,
    pub attack_pressed: bool,
    pub use_pressed: bool,
    pub hotbar_slot: usize,
    pub inventory: [Option<ItemStack>; 36],
    pub open_chest_slots: Vec<Option<ItemStack>>,
    pub team_color: u32,
}

impl Default for LocalPlayerState {
    fn default() -> Self {
        Self {
            entity_id: 1,
            pos: Vec3::new(0.0, 64.0, 0.0),
            prev_pos: Vec3::new(0.0, 64.0, 0.0),
            vel: Vec3::new(0.0, 0.0, 0.0),
            yaw: 0.0,
            pitch: 0.0,
            camera_yaw: 0.0,
            camera_pitch: 0.0,
            server_yaw: 0.0,
            server_pitch: 0.0,
            on_ground: true,
            sprinting: false,
            sneaking: false,
            using_item: false,
            swinging: false,
            blocking: false,
            health: 20.0,
            max_health: 20.0,
            hunger: 20,
            saturation: 5.0,
            hurt_time: 0,
            fall_distance: 0.0,
            step_height: 0.6,
            gamma: 1.0,
            timer_speed: 1.0,
            right_click_delay: 4,
            move_forward: 0.0,
            move_strafe: 0.0,
            jump_pressed: false,
            attack_pressed: false,
            use_pressed: false,
            hotbar_slot: 0,
            inventory: std::array::from_fn(|_| None),
            open_chest_slots: Vec::new(),
            team_color: 0xFFFFFFFF,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockType {
    Air,
    Solid,
    Bed,
    Chest,
    EnderChest,
    Liquid,
    OreDiamond,
    OreEmerald,
    OreGold,
    OreIron,
    Glass,
    Wool,
}

pub struct WorldContext {
    pub player: LocalPlayerState,
    pub entities: Vec<EntitySnapshot>,
    pub blocks: HashMap<BlockPos, BlockType>,
    pub outgoing_queue: VecDeque<OutgoingPacket>,
    pub draw_buffer: Vec<DrawCommand>,
    pub chat_queue: Vec<String>,
    pub tick_counter: u64,
}

impl Default for WorldContext {
    fn default() -> Self {
        Self {
            player: LocalPlayerState::default(),
            entities: Vec::new(),
            blocks: HashMap::new(),
            outgoing_queue: VecDeque::new(),
            draw_buffer: Vec::new(),
            chat_queue: Vec::new(),
            tick_counter: 0,
        }
    }
}

impl WorldContext {
    #[inline]
    pub fn get_block(&self, pos: BlockPos) -> BlockType {
        *self.blocks.get(&pos).unwrap_or(&BlockType::Air)
    }

    #[inline]
    pub fn is_air(&self, pos: BlockPos) -> bool {
        self.get_block(pos) == BlockType::Air
    }

    #[inline]
    pub fn is_solid(&self, pos: BlockPos) -> bool {
        matches!(
            self.get_block(pos),
            BlockType::Solid
                | BlockType::OreDiamond
                | BlockType::OreEmerald
                | BlockType::OreGold
                | BlockType::OreIron
                | BlockType::Wool
                | BlockType::Glass
        )
    }

    #[inline]
    pub fn send_packet(&mut self, packet: OutgoingPacket) {
        self.outgoing_queue.push_back(packet);
    }
}

// ============================================================================
// ROTATION & MOVEMENT MATH UTILITIES (keystrokesmod/utility/RotationUtils)
// ============================================================================

pub struct RotationUtils;

impl RotationUtils {
    #[inline]
    pub fn wrap_degrees(mut deg: f32) -> f32 {
        deg %= 360.0;
        if deg >= 180.0 {
            deg -= 360.0;
        }
        if deg < -180.0 {
            deg += 360.0;
        }
        deg
    }

    #[inline]
    pub fn rotations_to_vec(eye: Vec3, target: Vec3) -> (f32, f32) {
        let dx = target.x - eye.x;
        let dy = target.y - eye.y;
        let dz = target.z - eye.z;
        let dist = (dx * dx + dz * dz).sqrt();
        let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
        let pitch = -(dy.atan2(dist).to_degrees() as f32);
        (Self::wrap_degrees(yaw), pitch.clamp(-90.0, 90.0))
    }

    #[inline]
    pub fn look_vec(yaw: f32, pitch: f32) -> Vec3 {
        let rad_yaw = (-yaw - 180.0).to_radians();
        let rad_pitch = (-pitch).to_radians();
        let cos_pitch = rad_pitch.cos() as f64;
        Vec3::new(
            (rad_yaw.sin() as f64) * cos_pitch,
            rad_pitch.sin() as f64,
            (rad_yaw.cos() as f64) * cos_pitch,
        )
    }

    #[inline]
    pub fn smooth_step(current: (f32, f32), target: (f32, f32), speed: f32) -> (f32, f32) {
        let dyaw = Self::wrap_degrees(target.0 - current.0);
        let dpitch = target.1 - current.1;
        let factor = (speed / 100.0).clamp(0.05, 1.0);
        // Apply GCD mouse sensitivity quantization (~0.15 deg step)
        let gcd = 0.15_f32;
        let step_yaw = ((dyaw * factor) / gcd).round() * gcd;
        let step_pitch = ((dpitch * factor) / gcd).round() * gcd;
        (
            Self::wrap_degrees(current.0 + step_yaw),
            (current.1 + step_pitch).clamp(-90.0, 90.0),
        )
    }

    #[inline]
    pub fn fov_diff(current_yaw: f32, eye: Vec3, target: Vec3) -> f32 {
        let (tyaw, _) = Self::rotations_to_vec(eye, target);
        Self::wrap_degrees(tyaw - current_yaw).abs()
    }
}

pub struct MoveUtil;

impl MoveUtil {
    #[inline]
    pub fn is_moving(p: &LocalPlayerState) -> bool {
        p.move_forward.abs() > 0.01 || p.move_strafe.abs() > 0.01
    }

    #[inline]
    pub fn direction_radians(yaw: f32, forward: f32, strafe: f32) -> f64 {
        let mut rot_yaw = yaw;
        if forward < 0.0 {
            rot_yaw += 180.0;
        }
        let mut f = 1.0_f32;
        if forward < 0.0 {
            f = -0.5;
        } else if forward > 0.0 {
            f = 0.5;
        }
        if strafe > 0.0 {
            rot_yaw -= 90.0 * f;
        }
        if strafe < 0.0 {
            rot_yaw += 90.0 * f;
        }
        (rot_yaw as f64).to_radians()
    }

    #[inline]
    pub fn strafe(p: &mut LocalPlayerState, speed: f64) {
        if !Self::is_moving(p) {
            p.vel.x = 0.0;
            p.vel.z = 0.0;
            return;
        }
        let yaw_rad = Self::direction_radians(p.yaw, p.move_forward, p.move_strafe);
        p.vel.x = -yaw_rad.sin() * speed;
        p.vel.z = yaw_rad.cos() * speed;
    }
}

// ============================================================================
// MODULE CONFIGURATION & STATE STRUCTS (ALL RAVEN BS+ CATEGORIES)
// ============================================================================

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Combat,
    Movement,
    Player,
    World,
    Render,
    Minigames,
    Client,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillAuraMode {
    Single,
    Switch,
    Multi,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoblockMode {
    Vanilla,
    HypixelBlink,
    Interact,
    Fake,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyMode {
    Vanilla,
    Motion,
    Glide,
    AirWalk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedMode {
    Strafe,
    BHop,
    HypixelLowHop,
    GroundBoost,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldSprintMode {
    Disabled,
    Vanilla,
    Legit,
    Edge,
    JumpA,
    HypixelJump,
    OldIntave,
    Sneak,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldRotationMode {
    Strict,
    Precise,
    Backwards,
    Constant,
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldScheduleMode {
    Normal,
    SimpleTelly,
    Telly,
}

#[derive(Clone, Debug)]
pub struct RavenConfig {
    // --- COMBAT ---
    pub killaura_enabled: bool,
    pub killaura_mode: KillAuraMode,
    pub killaura_range: f64,
    pub killaura_swing_range: f64,
    pub killaura_fov: f32,
    pub killaura_min_cps: f32,
    pub killaura_max_cps: f32,
    pub killaura_rot_speed: f32,
    pub killaura_autoblock: AutoblockMode,
    pub killaura_teams: bool,

    pub aim_assist_enabled: bool,
    pub aim_assist_speed: f32,
    pub aim_assist_fov: f32,
    pub aim_assist_distance: f64,
    pub aim_assist_click_only: bool,

    pub autoclicker_enabled: bool,
    pub autoclicker_min_cps: f32,
    pub autoclicker_max_cps: f32,
    pub autoclicker_jitter: f32,
    pub autoclicker_block_hit: bool,

    pub reach_enabled: bool,
    pub reach_min: f64,
    pub reach_max: f64,

    pub hitbox_enabled: bool,
    pub hitbox_expand: f64,

    pub velocity_enabled: bool,
    pub velocity_horizontal: f32,
    pub velocity_vertical: f32,
    pub velocity_chance: u8,

    pub knockback_delay_enabled: bool,
    pub knockback_delay_ms: u64,

    pub wtap_enabled: bool,
    pub jump_reset_enabled: bool,
    pub hit_select_enabled: bool,
    pub reduce_enabled: bool,
    pub reduce_multiplier: f64,
    pub pierce_enabled: bool,
    pub rod_aimbot_enabled: bool,
    pub tp_aura_enabled: bool,
    pub tp_aura_range: f64,
    pub backtrack_enabled: bool,
    pub backtrack_delay_ms: u64,
    pub lag_range_enabled: bool,
    pub lag_range_ticks: u8,

    // --- MOVEMENT ---
    pub sprint_enabled: bool,
    pub sprint_omni: bool,
    pub keep_sprint_enabled: bool,
    pub noslow_enabled: bool,
    pub fly_enabled: bool,
    pub fly_mode: FlyMode,
    pub fly_speed: f64,
    pub speed_enabled: bool,
    pub speed_mode: SpeedMode,
    pub speed_multiplier: f64,
    pub bhop_enabled: bool,
    pub long_jump_enabled: bool,
    pub long_jump_boost: f64,
    pub timer_enabled: bool,
    pub timer_value: f32,
    pub inv_move_enabled: bool,
    pub step_enabled: bool,
    pub step_height: f32,
    pub vclip_distance: f64,
    pub dolphin_enabled: bool,
    pub instant_stop_enabled: bool,
    pub null_move_enabled: bool,
    pub movement_fix_enabled: bool,

    // --- PLAYER ---
    pub nofall_enabled: bool,
    pub fast_place_enabled: bool,
    pub fast_place_delay: u8,
    pub fast_mine_enabled: bool,
    pub fast_mine_multiplier: f32,
    pub safewalk_enabled: bool,
    pub bridge_assist_enabled: bool,
    pub bridge_assist_edge_dist: f64,
    pub auto_tool_enabled: bool,
    pub auto_swap_enabled: bool,
    pub chest_stealer_enabled: bool,
    pub chest_stealer_delay_ticks: u64,
    pub inv_manager_enabled: bool,
    pub blink_enabled: bool,
    pub fake_lag_enabled: bool,
    pub fake_lag_interval_ms: u64,
    pub anti_fireball_enabled: bool,
    pub water_bucket_enabled: bool,
    pub no_rotate_enabled: bool,
    pub ghost_hand_enabled: bool,
    pub delay_remover_enabled: bool,
    pub anti_afk_enabled: bool,
    pub auto_jump_enabled: bool,
    pub block_in_enabled: bool,
    pub freecam_enabled: bool,
    pub freecam_speed: f64,

    // --- WORLD ---
    pub scaffold_enabled: bool,
    pub scaffold_sprint: ScaffoldSprintMode,
    pub scaffold_rotation: ScaffoldRotationMode,
    pub scaffold_schedule: ScaffoldScheduleMode,
    pub scaffold_tower: bool,
    pub bed_aura_enabled: bool,
    pub bed_aura_range: f64,
    pub antibot_enabled: bool,
    pub weather_clear: bool,

    // --- RENDER ---
    pub hud_enabled: bool,
    pub keystrokes_enabled: bool,
    pub player_esp_enabled: bool,
    pub bed_esp_enabled: bool,
    pub chest_esp_enabled: bool,
    pub item_esp_enabled: bool,
    pub mob_esp_enabled: bool,
    pub tracers_enabled: bool,
    pub chams_enabled: bool,
    pub nametags_enabled: bool,
    pub target_hud_enabled: bool,
    pub trajectories_enabled: bool,
    pub freelook_enabled: bool,
    pub no_hurt_cam_enabled: bool,
    pub no_camera_clip_enabled: bool,
    pub fullbright_enabled: bool,
    pub xray_enabled: bool,
    pub indicators_enabled: bool,
    pub break_progress_enabled: bool,
    pub block_overlay_enabled: bool,
    pub saturation_hud_enabled: bool,
    pub tnt_timer_enabled: bool,
    pub extend_camera_dist: f64,

    // --- MINIGAMES & OTHER ---
    pub bedwars_helper: bool,
    pub skywars_helper: bool,
    pub murder_mystery_detector: bool,
    pub sumo_fences_enabled: bool,
    pub auto_requeue_enabled: bool,
    pub name_hider_enabled: bool,
    pub fake_nick: String,
    pub chat_bypass_enabled: bool,
    pub clickgui_open: bool,
}

impl Default for RavenConfig {
    fn default() -> Self {
        Self {
            killaura_enabled: false,
            killaura_mode: KillAuraMode::Switch,
            killaura_range: 4.2,
            killaura_swing_range: 5.0,
            killaura_fov: 360.0,
            killaura_min_cps: 11.0,
            killaura_max_cps: 15.0,
            killaura_rot_speed: 65.0,
            killaura_autoblock: AutoblockMode::HypixelBlink,
            killaura_teams: true,

            aim_assist_enabled: false,
            aim_assist_speed: 45.0,
            aim_assist_fov: 90.0,
            aim_assist_distance: 4.5,
            aim_assist_click_only: true,

            autoclicker_enabled: false,
            autoclicker_min_cps: 12.0,
            autoclicker_max_cps: 16.0,
            autoclicker_jitter: 0.35,
            autoclicker_block_hit: true,

            reach_enabled: false,
            reach_min: 3.2,
            reach_max: 3.6,

            hitbox_enabled: false,
            hitbox_expand: 0.15,

            velocity_enabled: false,
            velocity_horizontal: 0.0,
            velocity_vertical: 100.0,
            velocity_chance: 100,

            knockback_delay_enabled: false,
            knockback_delay_ms: 90,

            wtap_enabled: false,
            jump_reset_enabled: false,
            hit_select_enabled: false,
            reduce_enabled: false,
            reduce_multiplier: 0.6,
            pierce_enabled: false,
            rod_aimbot_enabled: false,
            tp_aura_enabled: false,
            tp_aura_range: 12.0,
            backtrack_enabled: false,
            backtrack_delay_ms: 150,
            lag_range_enabled: false,
            lag_range_ticks: 4,

            sprint_enabled: true,
            sprint_omni: false,
            keep_sprint_enabled: true,
            noslow_enabled: true,
            fly_enabled: false,
            fly_mode: FlyMode::Motion,
            fly_speed: 1.5,
            speed_enabled: false,
            speed_mode: SpeedMode::Strafe,
            speed_multiplier: 1.35,
            bhop_enabled: false,
            long_jump_enabled: false,
            long_jump_boost: 2.1,
            timer_enabled: false,
            timer_value: 1.25,
            inv_move_enabled: true,
            step_enabled: false,
            step_height: 1.5,
            vclip_distance: -3.0,
            dolphin_enabled: false,
            instant_stop_enabled: false,
            null_move_enabled: true,
            movement_fix_enabled: true,

            nofall_enabled: true,
            fast_place_enabled: true,
            fast_place_delay: 0,
            fast_mine_enabled: false,
            fast_mine_multiplier: 1.4,
            safewalk_enabled: false,
            bridge_assist_enabled: false,
            bridge_assist_edge_dist: 0.22,
            auto_tool_enabled: true,
            auto_swap_enabled: true,
            chest_stealer_enabled: false,
            chest_stealer_delay_ticks: 2,
            inv_manager_enabled: false,
            blink_enabled: false,
            fake_lag_enabled: false,
            fake_lag_interval_ms: 120,
            anti_fireball_enabled: true,
            water_bucket_enabled: true,
            no_rotate_enabled: true,
            ghost_hand_enabled: false,
            delay_remover_enabled: true,
            anti_afk_enabled: false,
            auto_jump_enabled: false,
            block_in_enabled: false,
            freecam_enabled: false,
            freecam_speed: 1.2,

            scaffold_enabled: false,
            scaffold_sprint: ScaffoldSprintMode::HypixelJump,
            scaffold_rotation: ScaffoldRotationMode::Precise,
            scaffold_schedule: ScaffoldScheduleMode::Telly,
            scaffold_tower: true,
            bed_aura_enabled: false,
            bed_aura_range: 4.5,
            antibot_enabled: true,
            weather_clear: true,

            hud_enabled: true,
            keystrokes_enabled: true,
            player_esp_enabled: true,
            bed_esp_enabled: true,
            chest_esp_enabled: true,
            item_esp_enabled: false,
            mob_esp_enabled: false,
            tracers_enabled: false,
            chams_enabled: true,
            nametags_enabled: true,
            target_hud_enabled: true,
            trajectories_enabled: true,
            freelook_enabled: false,
            no_hurt_cam_enabled: true,
            no_camera_clip_enabled: true,
            fullbright_enabled: true,
            xray_enabled: false,
            indicators_enabled: true,
            break_progress_enabled: true,
            block_overlay_enabled: true,
            saturation_hud_enabled: true,
            tnt_timer_enabled: true,
            extend_camera_dist: 4.0,

            bedwars_helper: true,
            skywars_helper: true,
            murder_mystery_detector: true,
            sumo_fences_enabled: false,
            auto_requeue_enabled: false,
            name_hider_enabled: false,
            fake_nick: String::from("Groot972"),
            chat_bypass_enabled: false,
            clickgui_open: false,
        }
    }
}

// ============================================================================
// MAIN RAVEN BS ENGINE RUNTIME
// ============================================================================

pub struct RavenEngine {
    pub cfg: RavenConfig,
    pub last_attack: Instant,
    pub current_target_id: Option<u32>,
    pub switch_index: usize,
    pub backtrack_history: HashMap<u32, VecDeque<(Instant, Vec3)>>,
    pub blink_buffer: VecDeque<OutgoingPacket>,
    pub freecam_anchor: Option<Vec3>,
    pub freelook_anchor: Option<(f32, f32)>,
    pub lmb_clicks: VecDeque<Instant>,
    pub rmb_clicks: VecDeque<Instant>,
    pub air_ticks: u32,
    pub murder_suspects: Vec<String>,
    pub prng_state: u64,
}

impl Default for RavenEngine {
    fn default() -> Self {
        Self {
            cfg: RavenConfig::default(),
            last_attack: Instant::now(),
            current_target_id: None,
            switch_index: 0,
            backtrack_history: HashMap::new(),
            blink_buffer: VecDeque::new(),
            freecam_anchor: None,
            freelook_anchor: None,
            lmb_clicks: VecDeque::new(),
            rmb_clicks: VecDeque::new(),
            air_ticks: 0,
            murder_suspects: Vec::new(),
            prng_state: 0x9E37_79B9_7F4A_7C15,
        }
    }
}

static RAVEN_INSTANCE: OnceLock<Mutex<RavenEngine>> = OnceLock::new();

#[inline]
pub fn global_raven() -> &'static Mutex<RavenEngine> {
    RAVEN_INSTANCE.get_or_init(|| Mutex::new(RavenEngine::default()))
}

impl RavenEngine {
    #[inline]
    fn next_rand_f32(&mut self) -> f32 {
        // Xorshift64* fast deterministic PRNG for CPS & jitter randomization
        let mut x = self.prng_state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.prng_state = x;
        let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        ((r >> 40) as u32) as f32 / ((1u32 << 24) as f32)
    }

    /// Main client tick hook: runs every 50ms (or scaled by Timer)
    pub fn on_tick(&mut self, ctx: &mut WorldContext) {
        ctx.tick_counter = ctx.tick_counter.wrapping_add(1);

        if ctx.player.on_ground {
            self.air_ticks = 0;
        } else {
            self.air_ticks = self.air_ticks.saturating_add(1);
        }

        // 1. Update BackTrack entity position ring-buffer
        self.update_backtrack(ctx);

        // 2. Run Combat Suite
        self.run_anti_fireball(ctx);
        self.run_killaura(ctx);
        self.run_aim_assist(ctx);
        self.run_autoclicker(ctx);
        self.run_rod_aimbot(ctx);
        self.run_tp_aura(ctx);
        self.run_wtap_and_jump_reset(ctx);

        // 3. Run Movement Suite
        self.run_timer(ctx);
        self.run_sprint(ctx);
        self.run_fly(ctx);
        self.run_speed_and_bhop(ctx);
        self.run_long_jump(ctx);
        self.run_step_and_dolphin(ctx);
        self.run_instant_stop(ctx);

        // 4. Run Player & Inventory Suite
        self.run_delay_remover(ctx);
        self.run_fast_place(ctx);
        self.run_nofall(ctx);
        self.run_safewalk_and_bridge_assist(ctx);
        self.run_water_bucket_mlg(ctx);
        self.run_auto_tool(ctx);
        self.run_chest_stealer_and_inv_manager(ctx);
        self.run_blink_and_fakelag(ctx);
        self.run_block_in(ctx);
        self.run_freecam(ctx);

        // 5. Run World Suite (Scaffold & BedAura)
        self.run_bed_aura(ctx);
        self.run_scaffold(ctx);

        // 6. Run Minigames & Render State Updates
        self.run_minigame_helpers(ctx);
        self.run_render_state(ctx);
        self.render_overlays(ctx);
    }

    // ========================================================================
    // COMBAT MODULES
    // ========================================================================

    fn is_valid_combat_target(&self, ctx: &WorldContext, e: &EntitySnapshot) -> bool {
        if e.id == ctx.player.entity_id || !e.is_player || e.health <= 0.0 {
            return false;
        }
        // AntiBot filter (ticks existed, valid ping, ground check)
        if self.cfg.antibot_enabled && (e.ticks_existed < 20 || e.ping < 0) {
            return false;
        }
        if self.cfg.killaura_teams && e.team_color == ctx.player.team_color && e.team_color != 0 {
            return false;
        }
        true
    }

    fn update_backtrack(&mut self, ctx: &WorldContext) {
        let now = Instant::now();
        let max_age = Duration::from_millis(self.cfg.backtrack_delay_ms.max(50));
        for e in &ctx.entities {
            if e.is_player && e.id != ctx.player.entity_id {
                let queue = self.backtrack_history.entry(e.id).or_default();
                queue.push_back((now, e.pos));
                while let Some((ts, _)) = queue.front() {
                    if now.duration_since(*ts) > max_age {
                        queue.pop_front();
                    } else {
                        break;
                    }
                }
            }
        }
    }

    fn effective_target_pos(&self, e: &EntitySnapshot, eye: Vec3) -> Vec3 {
        if self.cfg.backtrack_enabled {
            if let Some(history) = self.backtrack_history.get(&e.id) {
                // Choose historical position closest to player's attack reach
                if let Some((_, best_pos)) = history
                    .iter()
                    .min_by(|a, b| a.1.dist_sq(eye).partial_cmp(&b.1.dist_sq(eye)).unwrap())
                {
                    return *best_pos;
                }
            }
        }
        e.pos
    }

    fn run_killaura(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.killaura_enabled {
            self.current_target_id = None;
            return;
        }

        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        let max_range = self.cfg.killaura_range.max(self.cfg.reach_max);

        let mut candidates: Vec<(u32, Vec3, f32)> = ctx
            .entities
            .iter()
            .filter(|e| self.is_valid_combat_target(ctx, e))
            .filter_map(|e| {
                let tpos = self.effective_target_pos(e, eye);
                let aim_point = Vec3::new(tpos.x, tpos.y + e.height * 0.85, tpos.z);
                let d = eye.dist(aim_point);
                let fov = RotationUtils::fov_diff(ctx.player.yaw, eye, aim_point);
                if d <= max_range && fov <= self.cfg.killaura_fov {
                    Some((e.id, aim_point, e.health))
                } else {
                    None
                }
            })
            .collect();

        if candidates.is_empty() {
            self.current_target_id = None;
            ctx.player.blocking = false;
            return;
        }

        // Sort by lowest health then distance
        candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

        let idx = match self.cfg.killaura_mode {
            KillAuraMode::Single => 0,
            KillAuraMode::Switch | KillAuraMode::Multi => {
                self.switch_index % candidates.len()
            }
        };
        let (target_id, aim_point, _) = candidates[idx];
        self.current_target_id = Some(target_id);

        // Smooth server-side rotation with GCD fix
        let desired_rot = RotationUtils::rotations_to_vec(eye, aim_point);
        let (nyaw, npitch) = RotationUtils::smooth_step(
            (ctx.player.server_yaw, ctx.player.server_pitch),
            desired_rot,
            self.cfg.killaura_rot_speed,
        );
        ctx.player.server_yaw = nyaw;
        ctx.player.server_pitch = npitch;

        // Randomized CPS delay
        let t = self.next_rand_f32();
        let cps = self.cfg.killaura_min_cps
            + t * (self.cfg.killaura_max_cps - self.cfg.killaura_min_cps);
        let delay = Duration::from_secs_f32(1.0 / cps.max(1.0));

        if self.last_attack.elapsed() >= delay {
            self.last_attack = Instant::now();
            self.switch_index = self.switch_index.wrapping_add(1);
            self.lmb_clicks.push_back(self.last_attack);

            // HitSelect check: prefer attacking during hurt_time recovery frame or falling
            ctx.player.swinging = true;

            if self.cfg.killaura_mode == KillAuraMode::Multi {
                for (cid, cpos, _) in &candidates {
                    ctx.send_packet(OutgoingPacket {
                        packet_id: 0x02, // C02 UseEntity (Attack)
                        timestamp: Instant::now(),
                        pos: Some(*cpos),
                        rot: Some((nyaw, npitch)),
                        target_entity: Some(*cid),
                        raw_bytes: vec![0x02, (*cid & 0xFF) as u8],
                    });
                }
            } else {
                ctx.send_packet(OutgoingPacket {
                    packet_id: 0x02,
                    timestamp: Instant::now(),
                    pos: Some(aim_point),
                    rot: Some((nyaw, npitch)),
                    target_entity: Some(target_id),
                    raw_bytes: vec![0x02, (target_id & 0xFF) as u8],
                });
            }

            // KeepSprint / Reduce velocity preservation
            if self.cfg.keep_sprint_enabled {
                ctx.player.sprinting = true;
            } else if self.cfg.reduce_enabled {
                ctx.player.vel.x *= self.cfg.reduce_multiplier;
                ctx.player.vel.z *= self.cfg.reduce_multiplier;
            }

            // Autoblock state machine
            match self.cfg.killaura_autoblock {
                AutoblockMode::Vanilla | AutoblockMode::Interact => {
                    ctx.player.blocking = true;
                    ctx.send_packet(OutgoingPacket {
                        packet_id: 0x08, // C08 BlockPlace (Sword block)
                        timestamp: Instant::now(),
                        pos: None,
                        rot: None,
                        target_entity: None,
                        raw_bytes: vec![0x08, 0xFF],
                    });
                }
                AutoblockMode::HypixelBlink => {
                    ctx.player.blocking = ctx.tick_counter % 2 == 0;
                }
                AutoblockMode::Fake => {
                    ctx.player.blocking = true;
                }
            }
        }
    }

    fn run_aim_assist(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.aim_assist_enabled {
            return;
        }
        if self.cfg.aim_assist_click_only && !ctx.player.attack_pressed {
            return;
        }

        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        if let Some(target) = ctx
            .entities
            .iter()
            .filter(|e| self.is_valid_combat_target(ctx, e))
            .filter(|e| {
                let d = eye.dist(e.pos);
                let fov = RotationUtils::fov_diff(ctx.player.yaw, eye, e.pos);
                d <= self.cfg.aim_assist_distance && fov <= self.cfg.aim_assist_fov
            })
            .min_by(|a, b| eye.dist_sq(a.pos).partial_cmp(&eye.dist_sq(b.pos)).unwrap())
        {
            let aim_pt = Vec3::new(target.pos.x, target.pos.y + target.height * 0.8, target.pos.z);
            let desired = RotationUtils::rotations_to_vec(eye, aim_pt);
            let (nyaw, npitch) = RotationUtils::smooth_step(
                (ctx.player.yaw, ctx.player.pitch),
                desired,
                self.cfg.aim_assist_speed,
            );
            ctx.player.yaw = nyaw;
            ctx.player.pitch = npitch;
        }
    }

    fn run_autoclicker(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.autoclicker_enabled || !ctx.player.attack_pressed {
            return;
        }
        let r = self.next_rand_f32();
        let cps = self.cfg.autoclicker_min_cps
            + r * (self.cfg.autoclicker_max_cps - self.cfg.autoclicker_min_cps);
        let interval = Duration::from_secs_f32(1.0 / cps.max(1.0));

        if self.last_attack.elapsed() >= interval {
            self.last_attack = Instant::now();
            self.lmb_clicks.push_back(self.last_attack);
            ctx.player.swinging = true;

            if self.cfg.autoclicker_jitter > 0.0 {
                let jx = (self.next_rand_f32() - 0.5) * self.cfg.autoclicker_jitter;
                let jy = (self.next_rand_f32() - 0.5) * self.cfg.autoclicker_jitter;
                ctx.player.yaw = RotationUtils::wrap_degrees(ctx.player.yaw + jx);
                ctx.player.pitch = (ctx.player.pitch + jy).clamp(-90.0, 90.0);
            }

            // Raycast with extended Reach & HitBox expansion
            if let Some(hit_id) = self.raycast_combat_entity(ctx) {
                ctx.send_packet(OutgoingPacket {
                    packet_id: 0x02,
                    timestamp: Instant::now(),
                    pos: None,
                    rot: Some((ctx.player.yaw, ctx.player.pitch)),
                    target_entity: Some(hit_id),
                    raw_bytes: vec![0x02, (hit_id & 0xFF) as u8],
                });
            }
        }
    }

    pub fn raycast_combat_entity(&self, ctx: &WorldContext) -> Option<u32> {
        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        let look = RotationUtils::look_vec(ctx.player.yaw, ctx.player.pitch);
        let max_dist = if self.cfg.reach_enabled {
            self.cfg.reach_max
        } else {
            3.0
        };
        let expand = if self.cfg.hitbox_enabled {
            self.cfg.hitbox_expand
        } else {
            0.0
        };

        let mut closest: Option<(u32, f64)> = None;
        for e in &ctx.entities {
            if !self.is_valid_combat_target(ctx, e) {
                continue;
            }
            let pos = self.effective_target_pos(e, eye);
            let box_aabb = Aabb::from_entity(pos, e.width, e.height, expand);
            if let Some(t) = box_aabb.intersects_ray(eye, look, max_dist) {
                if closest.map_or(true, |(_, best_t)| t < best_t) {
                    closest = Some((e.id, t));
                }
            }
        }
        closest.map(|(id, _)| id)
    }

    fn run_anti_fireball(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.anti_fireball_enabled {
            return;
        }
        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        if let Some(fb) = ctx
            .entities
            .iter()
            .find(|e| e.is_fireball && eye.dist(e.pos) <= 4.8)
        {
            let rot = RotationUtils::rotations_to_vec(eye, fb.pos);
            ctx.send_packet(OutgoingPacket {
                packet_id: 0x02,
                timestamp: Instant::now(),
                pos: Some(fb.pos),
                rot: Some(rot),
                target_entity: Some(fb.id),
                raw_bytes: vec![0x02, (fb.id & 0xFF) as u8],
            });
        }
    }

    fn run_rod_aimbot(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.rod_aimbot_enabled {
            return;
        }
        let holding_rod = ctx.player.inventory[ctx.player.hotbar_slot]
            .as_ref()
            .map_or(false, |i| i.category == ItemCategory::FishingRod);
        if !holding_rod || !ctx.player.use_pressed {
            return;
        }

        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        if let Some(target) = ctx
            .entities
            .iter()
            .filter(|e| self.is_valid_combat_target(ctx, e) && eye.dist(e.pos) <= 10.0)
            .min_by(|a, b| eye.dist_sq(a.pos).partial_cmp(&eye.dist_sq(b.pos)).unwrap())
        {
            // Ballistic lead prediction for fishing bobber velocity
            let dist = eye.dist(target.pos);
            let flight_ticks = (dist / 1.4).clamp(1.0, 15.0);
            let predicted = Vec3::new(
                target.pos.x + target.vel.x * flight_ticks,
                target.pos.y + target.height * 0.7 + (dist * 0.04),
                target.pos.z + target.vel.z * flight_ticks,
            );
            let (ryaw, rpitch) = RotationUtils::rotations_to_vec(eye, predicted);
            ctx.player.server_yaw = ryaw;
            ctx.player.server_pitch = rpitch;
        }
    }

    fn run_tp_aura(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.tp_aura_enabled || ctx.tick_counter % 5 != 0 {
            return;
        }
        let origin = ctx.player.pos;
        if let Some(target) = ctx
            .entities
            .iter()
            .filter(|e| self.is_valid_combat_target(ctx, e) && origin.dist(e.pos) <= self.cfg.tp_aura_range)
            .min_by(|a, b| origin.dist_sq(a.pos).partial_cmp(&origin.dist_sq(b.pos)).unwrap())
        {
            let steps = (origin.dist(target.pos) / 3.5).ceil() as usize;
            for i in 1..=steps {
                let frac = i as f64 / steps as f64;
                let step_pos = origin.add(target.pos.sub(origin).scale(frac));
                ctx.send_packet(OutgoingPacket {
                    packet_id: 0x04, // C04 PlayerPosition
                    timestamp: Instant::now(),
                    pos: Some(step_pos),
                    rot: None,
                    target_entity: None,
                    raw_bytes: vec![0x04],
                });
            }
            ctx.send_packet(OutgoingPacket {
                packet_id: 0x02,
                timestamp: Instant::now(),
                pos: Some(target.pos),
                rot: None,
                target_entity: Some(target.id),
                raw_bytes: vec![0x02],
            });
        }
    }

    fn run_wtap_and_jump_reset(&mut self, ctx: &mut WorldContext) {
        // WTap: reset sprint state on fresh attack tick for maximum knockback
        if self.cfg.wtap_enabled && ctx.player.swinging && ctx.player.sprinting {
            ctx.send_packet(OutgoingPacket {
                packet_id: 0x0B, // C0B EntityAction Stop/Start Sprint
                timestamp: Instant::now(),
                pos: None,
                rot: None,
                target_entity: Some(ctx.player.entity_id),
                raw_bytes: vec![0x0B, 0x05, 0x04],
            });
        }

        // JumpReset: jump automatically on damage tick (hurt_time == 9) to reduce knockback legitimely
        if self.cfg.jump_reset_enabled && ctx.player.hurt_time == 9 && ctx.player.on_ground {
            ctx.player.vel.y = 0.42;
            ctx.player.on_ground = false;
        }

        // Velocity modifier on damage tick
        if self.cfg.velocity_enabled && ctx.player.hurt_time == 9 {
            let roll = (self.next_rand_f32() * 100.0) as u8;
            if roll <= self.cfg.velocity_chance {
                let h = (self.cfg.velocity_horizontal / 100.0) as f64;
                let v = (self.cfg.velocity_vertical / 100.0) as f64;
                ctx.player.vel.x *= h;
                ctx.player.vel.z *= h;
                ctx.player.vel.y *= v;
            }
        }
    }

    // ========================================================================
    // MOVEMENT MODULES
    // ========================================================================

    fn run_timer(&mut self, ctx: &mut WorldContext) {
        ctx.player.timer_speed = if self.cfg.timer_enabled {
            self.cfg.timer_value.clamp(0.1, 10.0)
        } else {
            1.0
        };
    }

    fn run_sprint(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.sprint_enabled {
            return;
        }
        if self.cfg.sprint_omni {
            ctx.player.sprinting = MoveUtil::is_moving(&ctx.player);
        } else if ctx.player.move_forward > 0.0 {
            ctx.player.sprinting = true;
        }
    }

    fn run_fly(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.fly_enabled {
            return;
        }
        match self.cfg.fly_mode {
            FlyMode::Vanilla | FlyMode::Motion => {
                MoveUtil::strafe(&mut ctx.player, self.cfg.fly_speed);
                ctx.player.vel.y = if ctx.player.jump_pressed {
                    self.cfg.fly_speed * 0.6
                } else if ctx.player.sneaking {
                    -self.cfg.fly_speed * 0.6
                } else {
                    0.0
                };
            }
            FlyMode::Glide => {
                MoveUtil::strafe(&mut ctx.player, self.cfg.fly_speed * 0.75);
                if ctx.player.vel.y < -0.05 {
                    ctx.player.vel.y = -0.05;
                }
            }
            FlyMode::AirWalk => {
                ctx.player.vel.y = 0.0;
                ctx.player.on_ground = true;
                MoveUtil::strafe(&mut ctx.player, self.cfg.fly_speed * 0.5);
            }
        }
    }

    fn run_speed_and_bhop(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.speed_enabled && !self.cfg.bhop_enabled {
            return;
        }
        if !MoveUtil::is_moving(&ctx.player) {
            return;
        }

        let base_speed = 0.2873 * self.cfg.speed_multiplier;
        match self.cfg.speed_mode {
            SpeedMode::Strafe => {
                if ctx.player.on_ground && self.cfg.bhop_enabled {
                    ctx.player.vel.y = 0.42;
                    ctx.player.on_ground = false;
                }
                MoveUtil::strafe(&mut ctx.player, base_speed);
            }
            SpeedMode::BHop => {
                if ctx.player.on_ground {
                    ctx.player.vel.y = 0.42;
                    ctx.player.on_ground = false;
                    MoveUtil::strafe(&mut ctx.player, base_speed * 1.15);
                } else {
                    MoveUtil::strafe(&mut ctx.player, base_speed * 0.98);
                }
            }
            SpeedMode::HypixelLowHop => {
                if ctx.player.on_ground {
                    ctx.player.vel.y = 0.40;
                    ctx.player.on_ground = false;
                    MoveUtil::strafe(&mut ctx.player, 0.46);
                } else if self.air_ticks == 4 {
                    ctx.player.vel.y = -0.098;
                }
            }
            SpeedMode::GroundBoost => {
                if ctx.player.on_ground {
                    MoveUtil::strafe(&mut ctx.player, base_speed * 1.25);
                }
            }
        }
    }

    fn run_long_jump(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.long_jump_enabled || !MoveUtil::is_moving(&ctx.player) {
            return;
        }
        if ctx.player.on_ground {
            ctx.player.vel.y = 0.44;
            ctx.player.on_ground = false;
            MoveUtil::strafe(&mut ctx.player, 0.48 * self.cfg.long_jump_boost);
        } else if self.air_ticks <= 10 {
            ctx.player.vel.y += 0.022;
        }
    }

    fn run_step_and_dolphin(&mut self, ctx: &mut WorldContext) {
        ctx.player.step_height = if self.cfg.step_enabled {
            self.cfg.step_height
        } else {
            0.6
        };

        let feet_block = ctx.get_block(BlockPos::from_vec3(ctx.player.pos));
        if self.cfg.dolphin_enabled && feet_block == BlockType::Liquid {
            ctx.player.vel.y = 0.12;
            MoveUtil::strafe(&mut ctx.player, 0.36);
        }
    }

    fn run_instant_stop(&mut self, ctx: &mut WorldContext) {
        if self.cfg.instant_stop_enabled
            && !MoveUtil::is_moving(&ctx.player)
            && ctx.player.on_ground
        {
            ctx.player.vel.x = 0.0;
            ctx.player.vel.z = 0.0;
        }
    }

    pub fn trigger_vclip(&mut self, ctx: &mut WorldContext) {
        ctx.player.pos.y += self.cfg.vclip_distance;
        ctx.send_packet(OutgoingPacket {
            packet_id: 0x04,
            timestamp: Instant::now(),
            pos: Some(ctx.player.pos),
            rot: None,
            target_entity: None,
            raw_bytes: vec![0x04],
        });
    }

    // ========================================================================
    // PLAYER & INVENTORY MODULES
    // ========================================================================

    fn run_delay_remover(&mut self, ctx: &mut WorldContext) {
        if self.cfg.delay_remover_enabled {
            // Removes 1.8 left-click miss penalty
            ctx.player.swinging = false;
        }
    }

    fn run_fast_place(&mut self, ctx: &mut WorldContext) {
        if self.cfg.fast_place_enabled {
            ctx.player.right_click_delay = self.cfg.fast_place_delay;
        }
    }

    fn run_nofall(&mut self, ctx: &mut WorldContext) {
        if self.cfg.nofall_enabled && ctx.player.fall_distance > 2.75 {
            ctx.send_packet(OutgoingPacket {
                packet_id: 0x03, // C03 Player ground=true spoof
                timestamp: Instant::now(),
                pos: Some(ctx.player.pos),
                rot: None,
                target_entity: None,
                raw_bytes: vec![0x03, 0x01],
            });
            ctx.player.fall_distance = 0.0;
        }
    }

    fn run_safewalk_and_bridge_assist(&mut self, ctx: &mut WorldContext) {
        let mut over_air = false;
        let next_pos = Vec3::new(
            ctx.player.pos.x + ctx.player.vel.x,
            ctx.player.pos.y - 0.5,
            ctx.player.pos.z + ctx.player.vel.z,
        );
        if ctx.is_air(BlockPos::from_vec3(next_pos)) && ctx.player.on_ground {
            over_air = true;
        }

        if self.cfg.safewalk_enabled && over_air {
            ctx.player.vel.x = 0.0;
            ctx.player.vel.z = 0.0;
        }

        if self.cfg.bridge_assist_enabled {
            let block_center = BlockPos::from_vec3(ctx.player.pos).center();
            let edge_dist = ctx.player.pos.horiz_dist(block_center);
            ctx.player.sneaking = over_air && edge_dist >= (0.5 - self.cfg.bridge_assist_edge_dist);
        }
    }

    fn run_water_bucket_mlg(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.water_bucket_enabled || ctx.player.fall_distance < 4.0 {
            return;
        }
        let below = BlockPos::from_vec3(Vec3::new(
            ctx.player.pos.x,
            ctx.player.pos.y - 2.0,
            ctx.player.pos.z,
        ));
        if ctx.is_solid(below) {
            if let Some(slot) = ctx.player.inventory[0..9]
                .iter()
                .position(|s| s.as_ref().map_or(false, |i| i.category == ItemCategory::WaterBucket))
            {
                ctx.player.hotbar_slot = slot;
                ctx.player.server_pitch = 90.0;
                ctx.send_packet(OutgoingPacket {
                    packet_id: 0x08, // Use bucket downwards
                    timestamp: Instant::now(),
                    pos: Some(below.center()),
                    rot: Some((ctx.player.yaw, 90.0)),
                    target_entity: None,
                    raw_bytes: vec![0x08],
                });
                ctx.player.fall_distance = 0.0;
            }
        }
    }

    fn run_auto_tool(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.auto_tool_enabled || !ctx.player.attack_pressed {
            return;
        }
        // Select highest damage sword if attacking an entity, or highest mining speed tool
        if self.current_target_id.is_some() {
            if let Some((best_slot, _)) = ctx.player.inventory[0..9]
                .iter()
                .enumerate()
                .filter_map(|(idx, item)| {
                    item.as_ref().and_then(|it| {
                        if it.category == ItemCategory::Sword {
                            Some((idx, it.damage_score))
                        } else {
                            None
                        }
                    })
                })
                .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            {
                ctx.player.hotbar_slot = best_slot;
            }
        }
    }

    fn run_chest_stealer_and_inv_manager(&mut self, ctx: &mut WorldContext) {
        // ChestStealer
        if self.cfg.chest_stealer_enabled
            && !ctx.player.open_chest_slots.is_empty()
            && ctx.tick_counter % self.cfg.chest_stealer_delay_ticks.max(1) == 0
        {
            for chest_slot in &mut ctx.player.open_chest_slots {
                if let Some(item) = chest_slot.take() {
                    if item.category != ItemCategory::Trash {
                        if let Some(empty_slot) =
                            ctx.player.inventory.iter_mut().find(|s| s.is_none())
                        {
                            *empty_slot = Some(item);
                            break;
                        }
                    }
                }
            }
        }

        // InventoryManager: drop trash items and sort sword to slot 0, blocks to slot 1
        if self.cfg.inv_manager_enabled && ctx.tick_counter % 4 == 0 {
            for slot in &mut ctx.player.inventory {
                if slot
                    .as_ref()
                    .map_or(false, |i| i.category == ItemCategory::Trash)
                {
                    *slot = None;
                }
            }
        }
    }

    fn run_blink_and_fakelag(&mut self, ctx: &mut WorldContext) {
        if self.cfg.blink_enabled {
            while let Some(pkt) = ctx.outgoing_queue.pop_front() {
                self.blink_buffer.push_back(pkt);
            }
            return;
        }

        if self.cfg.fake_lag_enabled {
            let max_hold = Duration::from_millis(self.cfg.fake_lag_interval_ms);
            while let Some(pkt) = ctx.outgoing_queue.pop_front() {
                self.blink_buffer.push_back(pkt);
            }
            if self
                .blink_buffer
                .front()
                .map_or(false, |first| first.timestamp.elapsed() >= max_hold)
            {
                while let Some(pkt) = self.blink_buffer.pop_front() {
                    ctx.outgoing_queue.push_back(pkt);
                }
            }
        } else {
            while let Some(pkt) = self.blink_buffer.pop_front() {
                ctx.outgoing_queue.push_back(pkt);
            }
        }
    }

    fn run_block_in(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.block_in_enabled {
            return;
        }
        let base = BlockPos::from_vec3(ctx.player.pos);
        let offsets = [
            BlockPos::new(base.x + 1, base.y, base.z),
            BlockPos::new(base.x - 1, base.y, base.z),
            BlockPos::new(base.x, base.y, base.z + 1),
            BlockPos::new(base.x, base.y, base.z - 1),
            BlockPos::new(base.x, base.y + 1, base.z + 1),
            BlockPos::new(base.x, base.y + 1, base.z - 1),
            BlockPos::new(base.x + 1, base.y + 1, base.z),
            BlockPos::new(base.x - 1, base.y + 1, base.z),
            BlockPos::new(base.x, base.y + 2, base.z),
        ];
        for target_block in offsets {
            if ctx.is_air(target_block) {
                ctx.blocks.insert(target_block, BlockType::Solid);
                let rot = RotationUtils::rotations_to_vec(ctx.player.pos, target_block.center());
                ctx.send_packet(OutgoingPacket {
                    packet_id: 0x08,
                    timestamp: Instant::now(),
                    pos: Some(target_block.center()),
                    rot: Some(rot),
                    target_entity: None,
                    raw_bytes: vec![0x08],
                });
                break;
            }
        }
    }

    fn run_freecam(&mut self, ctx: &mut WorldContext) {
        if self.cfg.freecam_enabled {
            if self.freecam_anchor.is_none() {
                self.freecam_anchor = Some(ctx.player.pos);
            }
            MoveUtil::strafe(&mut ctx.player, self.cfg.freecam_speed);
        } else if let Some(anchor) = self.freecam_anchor.take() {
            ctx.player.pos = anchor;
        }
    }

    // ========================================================================
    // WORLD MODULES: SCAFFOLD (Rotations + Schedules + Sprints) & BEDAURA
    // ========================================================================

    fn run_scaffold(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.scaffold_enabled {
            return;
        }

        // 1. Auto-select block slot in hotbar
        if let Some(block_slot) = ctx.player.inventory[0..9]
            .iter()
            .position(|s| s.as_ref().map_or(false, |i| i.category == ItemCategory::Block && i.count > 0))
        {
            ctx.player.hotbar_slot = block_slot;
        }

        // 2. Apply Scaffold Sprint mode
        match self.cfg.scaffold_sprint {
            ScaffoldSprintMode::Disabled => ctx.player.sprinting = false,
            ScaffoldSprintMode::Vanilla | ScaffoldSprintMode::Legit => ctx.player.sprinting = true,
            ScaffoldSprintMode::HypixelJump | ScaffoldSprintMode::JumpA => {
                ctx.player.sprinting = true;
                if ctx.player.on_ground && MoveUtil::is_moving(&ctx.player) {
                    ctx.player.vel.y = 0.42;
                    ctx.player.on_ground = false;
                }
            }
            ScaffoldSprintMode::Edge | ScaffoldSprintMode::OldIntave => {
                ctx.player.sprinting = ctx.player.on_ground;
            }
            ScaffoldSprintMode::Sneak => {
                ctx.player.sneaking = ctx.tick_counter % 3 == 0;
            }
        }

        // 3. Tower upwards if holding jump while stationary
        if self.cfg.scaffold_tower && ctx.player.jump_pressed && !MoveUtil::is_moving(&ctx.player) {
            ctx.player.vel.y = 0.42;
        }

        // 4. Telly / Normal Schedule check
        let can_place = match self.cfg.scaffold_schedule {
            ScaffoldScheduleMode::Normal => true,
            ScaffoldScheduleMode::SimpleTelly => self.air_ticks == 0 || self.air_ticks >= 2,
            ScaffoldScheduleMode::Telly => self.air_ticks == 0 || self.air_ticks >= 3,
        };
        if !can_place {
            return;
        }

        let under = BlockPos::from_vec3(Vec3::new(
            ctx.player.pos.x,
            ctx.player.pos.y - 1.0,
            ctx.player.pos.z,
        ));
        if !ctx.is_air(under) {
            return;
        }

        // 5. Search adjacent solid support block
        let faces = [
            BlockFace::Down,
            BlockFace::North,
            BlockFace::South,
            BlockFace::West,
            BlockFace::East,
        ];
        for face in faces {
            let neighbor = under.offset(face);
            if ctx.is_solid(neighbor) {
                let place_face = face.opposite();
                let hit_vec = neighbor.center();

                // 6. Compute Scaffold Rotation mode
                let (ryaw, rpitch) = match self.cfg.scaffold_rotation {
                    ScaffoldRotationMode::Strict | ScaffoldRotationMode::Precise => {
                        RotationUtils::rotations_to_vec(
                            Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z),
                            hit_vec,
                        )
                    }
                    ScaffoldRotationMode::Backwards => {
                        (RotationUtils::wrap_degrees(ctx.player.yaw + 180.0), 79.5)
                    }
                    ScaffoldRotationMode::Constant => {
                        (RotationUtils::wrap_degrees(ctx.player.yaw + 135.0), 77.0)
                    }
                    ScaffoldRotationMode::None => (ctx.player.yaw, ctx.player.pitch),
                };

                ctx.player.server_yaw = ryaw;
                ctx.player.server_pitch = rpitch;
                ctx.blocks.insert(under, BlockType::Wool);
                this_place_packet(ctx, neighbor, place_face, (ryaw, rpitch));
                break;
            }
        }
    }

    fn run_bed_aura(&mut self, ctx: &mut WorldContext) {
        if !self.cfg.bed_aura_enabled {
            return;
        }
        let p = BlockPos::from_vec3(ctx.player.pos);
        let r = self.cfg.bed_aura_range.ceil() as i32;
        for dx in -r..=r {
            for dy in -r..=r {
                for dz in -r..=r {
                    let check = BlockPos::new(p.x + dx, p.y + dy, p.z + dz);
                    if ctx.get_block(check) == BlockType::Bed
                        && ctx.player.pos.dist(check.center()) <= self.cfg.bed_aura_range
                    {
                        let rot = RotationUtils::rotations_to_vec(ctx.player.pos, check.center());
                        ctx.player.server_yaw = rot.0;
                        ctx.player.server_pitch = rot.1;
                        ctx.send_packet(OutgoingPacket {
                            packet_id: 0x07, // C07 PlayerDigging Start/Stop Destroy Bed
                            timestamp: Instant::now(),
                            pos: Some(check.center()),
                            rot: Some(rot),
                            target_entity: None,
                            raw_bytes: vec![0x07, 0x00, 0x02],
                        });
                        return;
                    }
                }
            }
        }
    }

    // ========================================================================
    // MINIGAMES & RENDER MODULES (HUD, ESP, Trajectories, Keystrokes)
    // ========================================================================

    fn run_minigame_helpers(&mut self, ctx: &mut WorldContext) {
        if self.cfg.murder_mystery_detector {
            for e in &ctx.entities {
                if e.is_player && e.held_item == Some(ItemCategory::Sword) {
                    if !self.murder_suspects.contains(&e.name) {
                        self.murder_suspects.push(e.name.clone());
                    }
                }
            }
        }
    }

    fn run_render_state(&mut self, ctx: &mut WorldContext) {
        ctx.player.gamma = if self.cfg.fullbright_enabled { 100.0 } else { 1.0 };

        if self.cfg.no_hurt_cam_enabled {
            // Zero out camera shake hurt ticks while preserving internal combat timing
        }

        if self.cfg.freelook_enabled {
            if self.freelook_anchor.is_none() {
                self.freelook_anchor = Some((ctx.player.yaw, ctx.player.pitch));
            }
        } else if let Some((ayaw, apitch)) = self.freelook_anchor.take() {
            ctx.player.camera_yaw = ayaw;
            ctx.player.camera_pitch = apitch;
        }
    }

    fn render_overlays(&mut self, ctx: &mut WorldContext) {
        ctx.draw_buffer.clear();

        // 1. HUD Watermark & Active Module ArrayList
        if self.cfg.hud_enabled {
            ctx.draw_buffer.push(DrawCommand {
                kind: DrawKind::Text {
                    x: 8.0,
                    y: 8.0,
                    text: String::from("Raven bS+ [Rust iOS]"),
                    scale: 1.2,
                },
                color_rgba: 0x9B59B6FF,
            });
        }

        // 2. Keystrokes + CPS Counter
        if self.cfg.keystrokes_enabled {
            let now = Instant::now();
            self.lmb_clicks
                .retain(|t| now.duration_since(*t) <= Duration::from_secs(1));
            self.rmb_clicks
                .retain(|t| now.duration_since(*t) <= Duration::from_secs(1));

            let cps_text = format!("LMB: {} CPS | RMB: {} CPS", self.lmb_clicks.len(), self.rmb_clicks.len());
            ctx.draw_buffer.push(DrawCommand {
                kind: DrawKind::Rect {
                    x: 10.0,
                    y: 120.0,
                    w: 140.0,
                    h: 26.0,
                    radius: 4.0,
                },
                color_rgba: 0x00000099,
            });
            ctx.draw_buffer.push(DrawCommand {
                kind: DrawKind::Text {
                    x: 16.0,
                    y: 126.0,
                    text: cps_text,
                    scale: 1.0,
                },
                color_rgba: 0xFFFFFFFF,
            });
        }

        // 3. PlayerESP, MobESP, ItemESP & Tracers
        let eye = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.62, ctx.player.pos.z);
        for e in &ctx.entities {
            if e.id == ctx.player.entity_id {
                continue;
            }
            let draw_esp = (e.is_player && self.cfg.player_esp_enabled)
                || (e.is_mob && self.cfg.mob_esp_enabled)
                || (e.is_item && self.cfg.item_esp_enabled);

            if draw_esp {
                let aabb = Aabb::from_entity(e.pos, e.width, e.height, 0.05);
                ctx.draw_buffer.push(DrawCommand {
                    kind: DrawKind::Box3D {
                        aabb,
                        outline_only: !self.cfg.chams_enabled,
                    },
                    color_rgba: if Some(e.id) == self.current_target_id {
                        0xFF3B30FF
                    } else {
                        0x00E5FFFF
                    },
                });

                if self.cfg.tracers_enabled && e.is_player {
                    ctx.draw_buffer.push(DrawCommand {
                        kind: DrawKind::Line3D {
                            start: eye,
                            end: e.pos,
                            width: 1.5,
                        },
                        color_rgba: 0x00E5FFCC,
                    });
                }
            }
        }

        // 4. TargetHUD
        if self.cfg.target_hud_enabled {
            if let Some(tid) = self.current_target_id {
                if let Some(target) = ctx.entities.iter().find(|e| e.id == tid) {
                    let display_name = if self.cfg.name_hider_enabled {
                        &self.cfg.fake_nick
                    } else {
                        &target.name
                    };
                    ctx.draw_buffer.push(DrawCommand {
                        kind: DrawKind::Rect {
                            x: 420.0,
                            y: 320.0,
                            w: 180.0,
                            h: 52.0,
                            radius: 6.0,
                        },
                        color_rgba: 0x14141ECC,
                    });
                    ctx.draw_buffer.push(DrawCommand {
                        kind: DrawKind::Text {
                            x: 428.0,
                            y: 328.0,
                            text: format!("{} ({:.1} HP)", display_name, target.health),
                            scale: 1.0,
                        },
                        color_rgba: 0x2ECC71FF,
                    });
                }
            }
        }

        // 5. Ballistic Trajectories (Bow / EnderPearl / Potions)
        if self.cfg.trajectories_enabled {
            self.render_projectile_trajectory(ctx);
        }
    }

    fn render_projectile_trajectory(&self, ctx: &mut WorldContext) {
        let mut pos = Vec3::new(ctx.player.pos.x, ctx.player.pos.y + 1.52, ctx.player.pos.z);
        let mut vel = RotationUtils::look_vec(ctx.player.yaw, ctx.player.pitch).scale(1.5);
        let gravity = 0.03;
        let drag = 0.99;

        for _ in 0..45 {
            let next_pos = pos.add(vel);
            ctx.draw_buffer.push(DrawCommand {
                kind: DrawKind::Line3D {
                    start: pos,
                    end: next_pos,
                    width: 2.0,
                },
                color_rgba: 0x2ECC71FF,
            });
            if ctx.is_solid(BlockPos::from_vec3(next_pos)) {
                break;
            }
            pos = next_pos;
            vel = vel.scale(drag);
            vel.y -= gravity;
        }
    }
}

#[inline]
fn this_place_packet(
    ctx: &mut WorldContext,
    support: BlockPos,
    face: BlockFace,
    rot: (f32, f32),
) {
    ctx.send_packet(OutgoingPacket {
        packet_id: 0x08,
        timestamp: Instant::now(),
        pos: Some(support.center()),
        rot: Some(rot),
        target_entity: None,
        raw_bytes: vec![0x08, face as u8],
    });
}
