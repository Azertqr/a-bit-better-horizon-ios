// File: crates/horizon-client/src/raven_bs.rs
//! Complete Raven bS+ (1.3.3+1) Engine & Native Touch ClickGUI for Horizon iOS
//! Includes all 7 categories, 78 modules, Hypixel/Telly Scaffold schedules, Minigames (SumoFences,
//! BedWars, SkyWars, MurderMystery, WoolWars, SpeedBuilders), and `.menu` chat command.

#![allow(dead_code, clippy::too_many_arguments, clippy::type_complexity)]

use crate::hud::{HudAtlas, HudBatch, rgba};
use std::collections::{HashMap, VecDeque};
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

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
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
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
        Vec3::new(
            self.x as f64 + 0.5,
            self.y as f64 + 0.5,
            self.z as f64 + 0.5,
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KillAuraMode {
    Single,
    Switch,
    Multi,
}
impl KillAuraMode {
    pub fn next(self) -> Self {
        match self {
            Self::Single => Self::Switch,
            Self::Switch => Self::Multi,
            Self::Multi => Self::Single,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Single => "Single",
            Self::Switch => "Switch",
            Self::Multi => "Multi",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutoblockMode {
    Vanilla,
    HypixelBlink,
    Interact,
    Fake,
}
impl AutoblockMode {
    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::HypixelBlink,
            Self::HypixelBlink => Self::Interact,
            Self::Interact => Self::Fake,
            Self::Fake => Self::Vanilla,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::HypixelBlink => "HypixelBlink",
            Self::Interact => "Interact",
            Self::Fake => "Fake",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyMode {
    Vanilla,
    Motion,
    Glide,
    AirWalk,
}
impl FlyMode {
    pub fn next(self) -> Self {
        match self {
            Self::Vanilla => Self::Motion,
            Self::Motion => Self::Glide,
            Self::Glide => Self::AirWalk,
            Self::AirWalk => Self::Vanilla,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Vanilla => "Vanilla",
            Self::Motion => "Motion",
            Self::Glide => "Glide",
            Self::AirWalk => "AirWalk",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpeedMode {
    Strafe,
    BHop,
    HypixelLowHop,
    GroundBoost,
}
impl SpeedMode {
    pub fn next(self) -> Self {
        match self {
            Self::Strafe => Self::BHop,
            Self::BHop => Self::HypixelLowHop,
            Self::HypixelLowHop => Self::GroundBoost,
            Self::GroundBoost => Self::Strafe,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strafe => "Strafe",
            Self::BHop => "BHop",
            Self::HypixelLowHop => "HypixelHop",
            Self::GroundBoost => "Ground",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldSprintMode {
    Disabled,
    Vanilla,
    Legit,
    Edge,
    JumpSprint,
    HypixelSprint,
    HypixelJumpSprint,
    HypixelJump2Sprint,
    HypixelJump3Sprint,
    OldIntaveSprint,
    SneakSprint,
}
impl ScaffoldSprintMode {
    pub fn next(self) -> Self {
        match self {
            Self::Disabled => Self::Vanilla,
            Self::Vanilla => Self::Legit,
            Self::Legit => Self::Edge,
            Self::Edge => Self::JumpSprint,
            Self::JumpSprint => Self::HypixelSprint,
            Self::HypixelSprint => Self::HypixelJumpSprint,
            Self::HypixelJumpSprint => Self::HypixelJump2Sprint,
            Self::HypixelJump2Sprint => Self::HypixelJump3Sprint,
            Self::HypixelJump3Sprint => Self::OldIntaveSprint,
            Self::OldIntaveSprint => Self::SneakSprint,
            Self::SneakSprint => Self::Disabled,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "Disabled",
            Self::Vanilla => "Vanilla",
            Self::Legit => "Legit",
            Self::Edge => "Edge",
            Self::JumpSprint => "Jump",
            Self::HypixelSprint => "Hypixel",
            Self::HypixelJumpSprint => "HypixelJump1",
            Self::HypixelJump2Sprint => "HypixelJump2",
            Self::HypixelJump3Sprint => "HypixelJump3",
            Self::OldIntaveSprint => "OldIntave",
            Self::SneakSprint => "Sneak",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldRotationMode {
    None,
    Constant,
    Backwards,
    Strict,
    Precise,
}
impl ScaffoldRotationMode {
    pub fn next(self) -> Self {
        match self {
            Self::None => Self::Constant,
            Self::Constant => Self::Backwards,
            Self::Backwards => Self::Strict,
            Self::Strict => Self::Precise,
            Self::Precise => Self::None,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Constant => "Constant",
            Self::Backwards => "Backwards",
            Self::Strict => "Strict",
            Self::Precise => "Precise",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScaffoldScheduleMode {
    Normal,
    SimpleTelly,
    Telly,
}
impl ScaffoldScheduleMode {
    pub fn next(self) -> Self {
        match self {
            Self::Normal => Self::SimpleTelly,
            Self::SimpleTelly => Self::Telly,
            Self::Telly => Self::Normal,
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::SimpleTelly => "SimpleTelly",
            Self::Telly => "Telly",
        }
    }
}

#[derive(Clone, Debug)]
pub struct RavenConfig {
    // COMBAT (19 modules)
    pub aim_assist_enabled: bool,
    pub aim_assist_speed: f32,
    pub aim_assist_fov: f32,
    pub aim_assist_distance: f64,
    pub autoclicker_enabled: bool,
    pub autoclicker_min_cps: f32,
    pub autoclicker_max_cps: f32,
    pub autoclicker_jitter: f32,
    pub autoblock_enabled: bool,
    pub autoblock_mode: AutoblockMode,
    pub velocity_enabled: bool,
    pub velocity_horizontal: f32,
    pub velocity_vertical: f32,
    pub velocity_chance: u8,
    pub backtrack_enabled: bool,
    pub backtrack_delay_ms: u64,
    pub click_assist_enabled: bool,
    pub displace_enabled: bool,
    pub hitbox_enabled: bool,
    pub hitbox_expand: f64,
    pub hit_select_enabled: bool,
    pub jump_reset_enabled: bool,
    pub killaura_enabled: bool,
    pub killaura_mode: KillAuraMode,
    pub killaura_range: f64,
    pub killaura_fov: f32,
    pub killaura_min_cps: f32,
    pub killaura_max_cps: f32,
    pub killaura_rot_speed: f32,
    pub knockback_delay_enabled: bool,
    pub knockback_delay_ms: u64,
    pub lag_range_enabled: bool,
    pub lag_range_ticks: u8,
    pub piercing_enabled: bool,
    pub reach_enabled: bool,
    pub reach_min: f64,
    pub reach_max: f64,
    pub reduce_enabled: bool,
    pub reduce_multiplier: f64,
    pub rod_aimbot_enabled: bool,
    pub tp_aura_enabled: bool,
    pub tp_aura_range: f64,
    pub wtap_enabled: bool,

    // MOVEMENT (18 modules)
    pub bhop_enabled: bool,
    pub boost_enabled: bool,
    pub dolphin_enabled: bool,
    pub fly_enabled: bool,
    pub fly_mode: FlyMode,
    pub fly_speed: f64,
    pub instant_stop_enabled: bool,
    pub inv_move_enabled: bool,
    pub jump_enabled: bool,
    pub keep_sprint_enabled: bool,
    pub long_jump_enabled: bool,
    pub long_jump_boost: f64,
    pub movement_fix_enabled: bool,
    pub noslow_enabled: bool,
    pub null_move_enabled: bool,
    pub speed_enabled: bool,
    pub speed_mode: SpeedMode,
    pub speed_multiplier: f64,
    pub sprint_enabled: bool,
    pub sprint_omni: bool,
    pub stasis_enabled: bool,
    pub step_enabled: bool,
    pub step_height: f32,
    pub stop_motion_enabled: bool,
    pub teleport_enabled: bool,
    pub timer_enabled: bool,
    pub timer_value: f32,
    pub vclip_enabled: bool,
    pub vclip_distance: f64,

    // PLAYER (20 modules)
    pub anti_afk_enabled: bool,
    pub anti_fireball_enabled: bool,
    pub auto_jump_enabled: bool,
    pub auto_swap_enabled: bool,
    pub auto_tool_enabled: bool,
    pub bed_aura_enabled: bool,
    pub bed_aura_range: f64,
    pub blink_enabled: bool,
    pub block_in_enabled: bool,
    pub bridge_assist_enabled: bool,
    pub bridge_assist_edge_dist: f64,
    pub chest_stealer_enabled: bool,
    pub chest_stealer_delay_ticks: u64,
    pub delay_remover_enabled: bool,
    pub fake_lag_enabled: bool,
    pub fake_lag_interval_ms: u64,
    pub fast_mine_enabled: bool,
    pub fast_mine_multiplier: f32,
    pub fast_place_enabled: bool,
    pub fast_place_delay: u8,
    pub freecam_enabled: bool,
    pub freecam_speed: f64,
    pub ghost_hand_enabled: bool,
    pub inv_manager_enabled: bool,
    pub nofall_enabled: bool,
    pub no_rotate_enabled: bool,
    pub safewalk_enabled: bool,
    pub water_bucket_enabled: bool,

    // WORLD (3 modules + all Hypixel/Telly sub-modes)
    pub antibot_enabled: bool,
    pub scaffold_enabled: bool,
    pub scaffold_sprint: ScaffoldSprintMode,
    pub scaffold_rotation: ScaffoldRotationMode,
    pub scaffold_schedule: ScaffoldScheduleMode,
    pub scaffold_tower: bool,
    pub weather_clear: bool,

    // RENDER (25 modules)
    pub animations_enabled: bool,
    pub anti_debuff_enabled: bool,
    pub anti_shuffle_enabled: bool,
    pub arrows_enabled: bool,
    pub bed_esp_enabled: bool,
    pub block_esp_enabled: bool,
    pub block_overlay_enabled: bool,
    pub break_progress_enabled: bool,
    pub chams_enabled: bool,
    pub chest_esp_enabled: bool,
    pub damage_tags_enabled: bool,
    pub damage_tint_enabled: bool,
    pub extend_camera_enabled: bool,
    pub extend_camera_dist: f64,
    pub fall_view_enabled: bool,
    pub freelook_enabled: bool,
    pub fullbright_enabled: bool,
    pub hud_enabled: bool,
    pub hit_particles_enabled: bool,
    pub holdlook_enabled: bool,
    pub indicators_enabled: bool,
    pub item_esp_enabled: bool,
    pub item_physics_enabled: bool,
    pub keystrokes_enabled: bool,
    pub mob_esp_enabled: bool,
    pub nametags_enabled: bool,
    pub no_camera_clip_enabled: bool,
    pub no_hurt_cam_enabled: bool,
    pub player_esp_enabled: bool,
    pub potion_hud_enabled: bool,
    pub radar_enabled: bool,
    pub saturation_hud_enabled: bool,
    pub tnt_timer_enabled: bool,
    pub target_hud_enabled: bool,
    pub tracers_enabled: bool,
    pub trajectories_enabled: bool,
    pub xray_enabled: bool,

    // MINIGAMES & OTHER (16 modules)
    pub auto_requeue_enabled: bool,
    pub auto_who_enabled: bool,
    pub bedwars_helper: bool,
    pub bridge_info_enabled: bool,
    pub duels_stats_enabled: bool,
    pub murder_mystery_detector: bool,
    pub skywars_helper: bool,
    pub speed_builders_helper: bool,
    pub sumo_fences_enabled: bool,
    pub sumo_fences_height: f32,
    pub wool_wars_helper: bool,
    pub anticheat_module_enabled: bool,
    pub chat_bypass_enabled: bool,
    pub fake_chat_enabled: bool,
    pub latency_alerts_enabled: bool,
    pub name_hider_enabled: bool,
    pub fake_nick: String,
    pub view_packets_enabled: bool,

    // CLICKGUI & FLOATING BUTTON STATE
    pub clickgui_open: bool,
    pub float_btn_x: f32,
    pub float_btn_y: f32,
    pub dragging_btn: bool,
    pub selected_module: &'static str,
    pub col_scroll: [usize; 7],
}

impl Default for RavenConfig {
    fn default() -> Self {
        Self {
            aim_assist_enabled: false,
            aim_assist_speed: 45.0,
            aim_assist_fov: 90.0,
            aim_assist_distance: 4.5,
            autoclicker_enabled: false,
            autoclicker_min_cps: 12.0,
            autoclicker_max_cps: 16.0,
            autoclicker_jitter: 0.3,
            autoblock_enabled: false,
            autoblock_mode: AutoblockMode::HypixelBlink,
            velocity_enabled: false,
            velocity_horizontal: 0.0,
            velocity_vertical: 100.0,
            velocity_chance: 100,
            backtrack_enabled: false,
            backtrack_delay_ms: 150,
            click_assist_enabled: false,
            displace_enabled: false,
            hitbox_enabled: false,
            hitbox_expand: 0.15,
            hit_select_enabled: false,
            jump_reset_enabled: false,
            killaura_enabled: false,
            killaura_mode: KillAuraMode::Switch,
            killaura_range: 4.2,
            killaura_fov: 360.0,
            killaura_min_cps: 11.0,
            killaura_max_cps: 15.0,
            killaura_rot_speed: 65.0,
            knockback_delay_enabled: false,
            knockback_delay_ms: 90,
            lag_range_enabled: false,
            lag_range_ticks: 4,
            piercing_enabled: false,
            reach_enabled: false,
            reach_min: 3.2,
            reach_max: 3.8,
            reduce_enabled: false,
            reduce_multiplier: 0.6,
            rod_aimbot_enabled: false,
            tp_aura_enabled: false,
            tp_aura_range: 10.0,
            wtap_enabled: false,

            bhop_enabled: false,
            boost_enabled: false,
            dolphin_enabled: false,
            fly_enabled: false,
            fly_mode: FlyMode::Motion,
            fly_speed: 1.5,
            instant_stop_enabled: false,
            inv_move_enabled: false,
            jump_enabled: false,
            keep_sprint_enabled: false,
            long_jump_enabled: false,
            long_jump_boost: 2.0,
            movement_fix_enabled: false,
            noslow_enabled: false,
            null_move_enabled: false,
            speed_enabled: false,
            speed_mode: SpeedMode::HypixelLowHop,
            speed_multiplier: 1.35,
            sprint_enabled: false,
            sprint_omni: false,
            stasis_enabled: false,
            step_enabled: false,
            step_height: 1.5,
            stop_motion_enabled: false,
            teleport_enabled: false,
            timer_enabled: false,
            timer_value: 1.25,
            vclip_enabled: false,
            vclip_distance: -3.0,

            anti_afk_enabled: false,
            anti_fireball_enabled: false,
            auto_jump_enabled: false,
            auto_swap_enabled: false,
            auto_tool_enabled: false,
            bed_aura_enabled: false,
            bed_aura_range: 4.5,
            blink_enabled: false,
            block_in_enabled: false,
            bridge_assist_enabled: false,
            bridge_assist_edge_dist: 0.22,
            chest_stealer_enabled: false,
            chest_stealer_delay_ticks: 2,
            delay_remover_enabled: false,
            fake_lag_enabled: false,
            fake_lag_interval_ms: 120,
            fast_mine_enabled: false,
            fast_mine_multiplier: 1.4,
            fast_place_enabled: false,
            fast_place_delay: 0,
            freecam_enabled: false,
            freecam_speed: 1.2,
            ghost_hand_enabled: false,
            inv_manager_enabled: false,
            nofall_enabled: false,
            no_rotate_enabled: false,
            safewalk_enabled: false,
            water_bucket_enabled: false,

            antibot_enabled: false,
            scaffold_enabled: false,
            scaffold_sprint: ScaffoldSprintMode::HypixelJump2Sprint,
            scaffold_rotation: ScaffoldRotationMode::Precise,
            scaffold_schedule: ScaffoldScheduleMode::Telly,
            scaffold_tower: false,
            weather_clear: false,

            animations_enabled: false,
            anti_debuff_enabled: false,
            anti_shuffle_enabled: false,
            arrows_enabled: false,
            bed_esp_enabled: false,
            block_esp_enabled: false,
            block_overlay_enabled: false,
            break_progress_enabled: false,
            chams_enabled: false,
            chest_esp_enabled: false,
            damage_tags_enabled: false,
            damage_tint_enabled: false,
            extend_camera_enabled: false,
            extend_camera_dist: 4.0,
            fall_view_enabled: false,
            freelook_enabled: false,
            fullbright_enabled: false,
            hud_enabled: false,
            hit_particles_enabled: false,
            holdlook_enabled: false,
            indicators_enabled: false,
            item_esp_enabled: false,
            item_physics_enabled: false,
            keystrokes_enabled: false,
            mob_esp_enabled: false,
            nametags_enabled: false,
            no_camera_clip_enabled: false,
            no_hurt_cam_enabled: false,
            player_esp_enabled: false,
            potion_hud_enabled: false,
            radar_enabled: false,
            saturation_hud_enabled: false,
            tnt_timer_enabled: false,
            target_hud_enabled: false,
            tracers_enabled: false,
            trajectories_enabled: false,
            xray_enabled: false,

            auto_requeue_enabled: false,
            auto_who_enabled: false,
            bedwars_helper: false,
            bridge_info_enabled: false,
            duels_stats_enabled: false,
            murder_mystery_detector: false,
            skywars_helper: false,
            speed_builders_helper: false,
            sumo_fences_enabled: false,
            sumo_fences_height: 2.0,
            wool_wars_helper: false,
            anticheat_module_enabled: false,
            chat_bypass_enabled: false,
            fake_chat_enabled: false,
            latency_alerts_enabled: false,
            name_hider_enabled: false,
            fake_nick: String::from("Groot972"),
            view_packets_enabled: false,

            clickgui_open: false,
            float_btn_x: 6.0,
            float_btn_y: 42.0,
            dragging_btn: false,
            selected_module: "FastPlace",
            col_scroll: [0; 7],
        }
    }
}

pub struct RavenEngine {
    pub cfg: RavenConfig,
    pub last_attack: Instant,
    pub lmb_clicks: VecDeque<Instant>,
    pub rmb_clicks: VecDeque<Instant>,
}

impl Default for RavenEngine {
    fn default() -> Self {
        Self {
            cfg: RavenConfig::default(),
            last_attack: Instant::now(),
            lmb_clicks: VecDeque::new(),
            rmb_clicks: VecDeque::new(),
        }
    }
}

static RAVEN_INSTANCE: OnceLock<Mutex<RavenEngine>> = OnceLock::new();

#[inline]
pub fn global_raven() -> &'static Mutex<RavenEngine> {
    RAVEN_INSTANCE.get_or_init(|| Mutex::new(RavenEngine::default()))
}

const CATEGORIES: [&str; 7] = [
    "Combat",
    "Movement",
    "Player",
    "World",
    "Render",
    "Minigames",
    "Other",
];

fn category_modules(cat: usize) -> &'static [&'static str] {
    match cat {
        0 => &[
            "AimAssist",
            "AutoClicker",
            "Autoblock",
            "Velocity",
            "BackTrack",
            "ClickAssist",
            "Displace",
            "HitBox",
            "HitSelect",
            "JumpReset",
            "KillAura",
            "KBDelay",
            "LagRange",
            "Piercing",
            "Reach",
            "Reduce",
            "RodAimbot",
            "TPAura",
            "WTap",
        ],
        1 => &[
            "BHop",
            "Boost",
            "Dolphin",
            "Fly",
            "InstantStop",
            "InvMove",
            "Jump",
            "KeepSprint",
            "LongJump",
            "MovementFix",
            "NoSlow",
            "NullMove",
            "Speed",
            "Sprint",
            "Stasis",
            "Step",
            "StopMotion",
            "Teleport",
            "Timer",
            "VClip",
        ],
        2 => &[
            "AntiAFK",
            "AntiFireball",
            "AutoJump",
            "AutoSwap",
            "AutoTool",
            "BedAura",
            "Blink",
            "BlockIn",
            "BridgeAssist",
            "ChestStealer",
            "DelayRemover",
            "FakeLag",
            "FastMine",
            "FastPlace",
            "Freecam",
            "GhostHand",
            "InvManager",
            "NoFall",
            "NoRotate",
            "SafeWalk",
            "WaterBucket",
        ],
        3 => &["AntiBot", "Scaffold", "Weather"],
        4 => &[
            "Animations",
            "AntiDebuff",
            "AntiShuffle",
            "Arrows",
            "BedESP",
            "BlockESP",
            "BlockOverlay",
            "BreakProgress",
            "Chams",
            "ChestESP",
            "DamageTags",
            "DamageTint",
            "ExtendCamera",
            "FallView",
            "Freelook",
            "Fullbright",
            "HUD",
            "HitParticles",
            "Holdlook",
            "Indicators",
            "ItemESP",
            "ItemPhysics",
            "Keystrokes",
            "MobESP",
            "Nametags",
            "NoCameraClip",
            "NoHurtCam",
            "PlayerESP",
            "PotionHUD",
            "Radar",
            "Saturation",
            "TNTTimer",
            "TargetHUD",
            "Tracers",
            "Trajectories",
            "Xray",
        ],
        5 => &[
            "AutoRequeue",
            "AutoWho",
            "BedWars",
            "BridgeInfo",
            "DuelsStats",
            "MurderMystery",
            "SkyWars",
            "SpeedBuilders",
            "SumoFences",
            "WoolWars",
        ],
        _ => &[
            "Anticheat",
            "ChatBypass",
            "FakeChat",
            "LatencyAlerts",
            "NameHider",
            "ViewPackets",
        ],
    }
}

impl RavenConfig {
    pub fn is_enabled(&self, name: &str) -> bool {
        match name {
            "AimAssist" => self.aim_assist_enabled,
            "AutoClicker" => self.autoclicker_enabled,
            "Autoblock" => self.autoblock_enabled,
            "Velocity" => self.velocity_enabled,
            "BackTrack" => self.backtrack_enabled,
            "ClickAssist" => self.click_assist_enabled,
            "Displace" => self.displace_enabled,
            "HitBox" => self.hitbox_enabled,
            "HitSelect" => self.hit_select_enabled,
            "JumpReset" => self.jump_reset_enabled,
            "KillAura" => self.killaura_enabled,
            "KBDelay" => self.knockback_delay_enabled,
            "LagRange" => self.lag_range_enabled,
            "Piercing" => self.piercing_enabled,
            "Reach" => self.reach_enabled,
            "Reduce" => self.reduce_enabled,
            "RodAimbot" => self.rod_aimbot_enabled,
            "TPAura" => self.tp_aura_enabled,
            "WTap" => self.wtap_enabled,

            "BHop" => self.bhop_enabled,
            "Boost" => self.boost_enabled,
            "Dolphin" => self.dolphin_enabled,
            "Fly" => self.fly_enabled,
            "InstantStop" => self.instant_stop_enabled,
            "InvMove" => self.inv_move_enabled,
            "Jump" => self.jump_enabled,
            "KeepSprint" => self.keep_sprint_enabled,
            "LongJump" => self.long_jump_enabled,
            "MovementFix" => self.movement_fix_enabled,
            "NoSlow" => self.noslow_enabled,
            "NullMove" => self.null_move_enabled,
            "Speed" => self.speed_enabled,
            "Sprint" => self.sprint_enabled,
            "Stasis" => self.stasis_enabled,
            "Step" => self.step_enabled,
            "StopMotion" => self.stop_motion_enabled,
            "Teleport" => self.teleport_enabled,
            "Timer" => self.timer_enabled,
            "VClip" => self.vclip_enabled,

            "AntiAFK" => self.anti_afk_enabled,
            "AntiFireball" => self.anti_fireball_enabled,
            "AutoJump" => self.auto_jump_enabled,
            "AutoSwap" => self.auto_swap_enabled,
            "AutoTool" => self.auto_tool_enabled,
            "BedAura" => self.bed_aura_enabled,
            "Blink" => self.blink_enabled,
            "BlockIn" => self.block_in_enabled,
            "BridgeAssist" => self.bridge_assist_enabled,
            "ChestStealer" => self.chest_stealer_enabled,
            "DelayRemover" => self.delay_remover_enabled,
            "FakeLag" => self.fake_lag_enabled,
            "FastMine" => self.fast_mine_enabled,
            "FastPlace" => self.fast_place_enabled,
            "Freecam" => self.freecam_enabled,
            "GhostHand" => self.ghost_hand_enabled,
            "InvManager" => self.inv_manager_enabled,
            "NoFall" => self.nofall_enabled,
            "NoRotate" => self.no_rotate_enabled,
            "SafeWalk" => self.safewalk_enabled,
            "WaterBucket" => self.water_bucket_enabled,

            "AntiBot" => self.antibot_enabled,
            "Scaffold" => self.scaffold_enabled,
            "Weather" => self.weather_clear,

            "Animations" => self.animations_enabled,
            "AntiDebuff" => self.anti_debuff_enabled,
            "AntiShuffle" => self.anti_shuffle_enabled,
            "Arrows" => self.arrows_enabled,
            "BedESP" => self.bed_esp_enabled,
            "BlockESP" => self.block_esp_enabled,
            "BlockOverlay" => self.block_overlay_enabled,
            "BreakProgress" => self.break_progress_enabled,
            "Chams" => self.chams_enabled,
            "ChestESP" => self.chest_esp_enabled,
            "DamageTags" => self.damage_tags_enabled,
            "DamageTint" => self.damage_tint_enabled,
            "ExtendCamera" => self.extend_camera_enabled,
            "FallView" => self.fall_view_enabled,
            "Freelook" => self.freelook_enabled,
            "Fullbright" => self.fullbright_enabled,
            "HUD" => self.hud_enabled,
            "HitParticles" => self.hit_particles_enabled,
            "Holdlook" => self.holdlook_enabled,
            "Indicators" => self.indicators_enabled,
            "ItemESP" => self.item_esp_enabled,
            "ItemPhysics" => self.item_physics_enabled,
            "Keystrokes" => self.keystrokes_enabled,
            "MobESP" => self.mob_esp_enabled,
            "Nametags" => self.nametags_enabled,
            "NoCameraClip" => self.no_camera_clip_enabled,
            "NoHurtCam" => self.no_hurt_cam_enabled,
            "PlayerESP" => self.player_esp_enabled,
            "PotionHUD" => self.potion_hud_enabled,
            "Radar" => self.radar_enabled,
            "Saturation" => self.saturation_hud_enabled,
            "TNTTimer" => self.tnt_timer_enabled,
            "TargetHUD" => self.target_hud_enabled,
            "Tracers" => self.tracers_enabled,
            "Trajectories" => self.trajectories_enabled,
            "Xray" => self.xray_enabled,

            "AutoRequeue" => self.auto_requeue_enabled,
            "AutoWho" => self.auto_who_enabled,
            "BedWars" => self.bedwars_helper,
            "BridgeInfo" => self.bridge_info_enabled,
            "DuelsStats" => self.duels_stats_enabled,
            "MurderMystery" => self.murder_mystery_detector,
            "SkyWars" => self.skywars_helper,
            "SpeedBuilders" => self.speed_builders_helper,
            "SumoFences" => self.sumo_fences_enabled,
            "WoolWars" => self.wool_wars_helper,

            "Anticheat" => self.anticheat_module_enabled,
            "ChatBypass" => self.chat_bypass_enabled,
            "FakeChat" => self.fake_chat_enabled,
            "LatencyAlerts" => self.latency_alerts_enabled,
            "NameHider" => self.name_hider_enabled,
            "ViewPackets" => self.view_packets_enabled,
            _ => false,
        }
    }

    pub fn toggle(&mut self, name: &str) -> bool {
        let state = match name {
            "AimAssist" => { self.aim_assist_enabled = !self.aim_assist_enabled; self.aim_assist_enabled }
            "AutoClicker" => { self.autoclicker_enabled = !self.autoclicker_enabled; self.autoclicker_enabled }
            "Autoblock" => { self.autoblock_enabled = !self.autoblock_enabled; self.autoblock_enabled }
            "Velocity" => { self.velocity_enabled = !self.velocity_enabled; self.velocity_enabled }
            "BackTrack" => { self.backtrack_enabled = !self.backtrack_enabled; self.backtrack_enabled }
            "ClickAssist" => { self.click_assist_enabled = !self.click_assist_enabled; self.click_assist_enabled }
            "Displace" => { self.displace_enabled = !self.displace_enabled; self.displace_enabled }
            "HitBox" => { self.hitbox_enabled = !self.hitbox_enabled; self.hitbox_enabled }
            "HitSelect" => { self.hit_select_enabled = !self.hit_select_enabled; self.hit_select_enabled }
            "JumpReset" => { self.jump_reset_enabled = !self.jump_reset_enabled; self.jump_reset_enabled }
            "KillAura" => { self.killaura_enabled = !self.killaura_enabled; self.killaura_enabled }
            "KBDelay" => { self.knockback_delay_enabled = !self.knockback_delay_enabled; self.knockback_delay_enabled }
            "LagRange" => { self.lag_range_enabled = !self.lag_range_enabled; self.lag_range_enabled }
            "Piercing" => { self.piercing_enabled = !self.piercing_enabled; self.piercing_enabled }
            "Reach" => { self.reach_enabled = !self.reach_enabled; self.reach_enabled }
            "Reduce" => { self.reduce_enabled = !self.reduce_enabled; self.reduce_enabled }
            "RodAimbot" => { self.rod_aimbot_enabled = !self.rod_aimbot_enabled; self.rod_aimbot_enabled }
            "TPAura" => { self.tp_aura_enabled = !self.tp_aura_enabled; self.tp_aura_enabled }
            "WTap" => { self.wtap_enabled = !self.wtap_enabled; self.wtap_enabled }

            "BHop" => { self.bhop_enabled = !self.bhop_enabled; self.bhop_enabled }
            "Boost" => { self.boost_enabled = !self.boost_enabled; self.boost_enabled }
            "Dolphin" => { self.dolphin_enabled = !self.dolphin_enabled; self.dolphin_enabled }
            "Fly" => { self.fly_enabled = !self.fly_enabled; self.fly_enabled }
            "InstantStop" => { self.instant_stop_enabled = !self.instant_stop_enabled; self.instant_stop_enabled }
            "InvMove" => { self.inv_move_enabled = !self.inv_move_enabled; self.inv_move_enabled }
            "Jump" => { self.jump_enabled = !self.jump_enabled; self.jump_enabled }
            "KeepSprint" => { self.keep_sprint_enabled = !self.keep_sprint_enabled; self.keep_sprint_enabled }
            "LongJump" => { self.long_jump_enabled = !self.long_jump_enabled; self.long_jump_enabled }
            "MovementFix" => { self.movement_fix_enabled = !self.movement_fix_enabled; self.movement_fix_enabled }
            "NoSlow" => { self.noslow_enabled = !self.noslow_enabled; self.noslow_enabled }
            "NullMove" => { self.null_move_enabled = !self.null_move_enabled; self.null_move_enabled }
            "Speed" => { self.speed_enabled = !self.speed_enabled; self.speed_enabled }
            "Sprint" => { self.sprint_enabled = !self.sprint_enabled; self.sprint_enabled }
            "Stasis" => { self.stasis_enabled = !self.stasis_enabled; self.stasis_enabled }
            "Step" => { self.step_enabled = !self.step_enabled; self.step_enabled }
            "StopMotion" => { self.stop_motion_enabled = !self.stop_motion_enabled; self.stop_motion_enabled }
            "Teleport" => { self.teleport_enabled = !self.teleport_enabled; self.teleport_enabled }
            "Timer" => { self.timer_enabled = !self.timer_enabled; self.timer_enabled }
            "VClip" => { self.vclip_enabled = !self.vclip_enabled; self.vclip_enabled }

            "AntiAFK" => { self.anti_afk_enabled = !self.anti_afk_enabled; self.anti_afk_enabled }
            "AntiFireball" => { self.anti_fireball_enabled = !self.anti_fireball_enabled; self.anti_fireball_enabled }
            "AutoJump" => { self.auto_jump_enabled = !self.auto_jump_enabled; self.auto_jump_enabled }
            "AutoSwap" => { self.auto_swap_enabled = !self.auto_swap_enabled; self.auto_swap_enabled }
            "AutoTool" => { self.auto_tool_enabled = !self.auto_tool_enabled; self.auto_tool_enabled }
            "BedAura" => { self.bed_aura_enabled = !self.bed_aura_enabled; self.bed_aura_enabled }
            "Blink" => { self.blink_enabled = !self.blink_enabled; self.blink_enabled }
            "BlockIn" => { self.block_in_enabled = !self.block_in_enabled; self.block_in_enabled }
            "BridgeAssist" => { self.bridge_assist_enabled = !self.bridge_assist_enabled; self.bridge_assist_enabled }
            "ChestStealer" => { self.chest_stealer_enabled = !self.chest_stealer_enabled; self.chest_stealer_enabled }
            "DelayRemover" => { self.delay_remover_enabled = !self.delay_remover_enabled; self.delay_remover_enabled }
            "FakeLag" => { self.fake_lag_enabled = !self.fake_lag_enabled; self.fake_lag_enabled }
            "FastMine" => { self.fast_mine_enabled = !self.fast_mine_enabled; self.fast_mine_enabled }
            "FastPlace" => { self.fast_place_enabled = !self.fast_place_enabled; self.fast_place_enabled }
            "Freecam" => { self.freecam_enabled = !self.freecam_enabled; self.freecam_enabled }
            "GhostHand" => { self.ghost_hand_enabled = !self.ghost_hand_enabled; self.ghost_hand_enabled }
            "InvManager" => { self.inv_manager_enabled = !self.inv_manager_enabled; self.inv_manager_enabled }
            "NoFall" => { self.nofall_enabled = !self.nofall_enabled; self.nofall_enabled }
            "NoRotate" => { self.no_rotate_enabled = !self.no_rotate_enabled; self.no_rotate_enabled }
            "SafeWalk" => { self.safewalk_enabled = !self.safewalk_enabled; self.safewalk_enabled }
            "WaterBucket" => { self.water_bucket_enabled = !self.water_bucket_enabled; self.water_bucket_enabled }

            "AntiBot" => { self.antibot_enabled = !self.antibot_enabled; self.antibot_enabled }
            "Scaffold" => { self.scaffold_enabled = !self.scaffold_enabled; self.scaffold_enabled }
            "Weather" => { self.weather_clear = !self.weather_clear; self.weather_clear }

            "Animations" => { self.animations_enabled = !self.animations_enabled; self.animations_enabled }
            "AntiDebuff" => { self.anti_debuff_enabled = !self.anti_debuff_enabled; self.anti_debuff_enabled }
            "AntiShuffle" => { self.anti_shuffle_enabled = !self.anti_shuffle_enabled; self.anti_shuffle_enabled }
            "Arrows" => { self.arrows_enabled = !self.arrows_enabled; self.arrows_enabled }
            "BedESP" => { self.bed_esp_enabled = !self.bed_esp_enabled; self.bed_esp_enabled }
            "BlockESP" => { self.block_esp_enabled = !self.block_esp_enabled; self.block_esp_enabled }
            "BlockOverlay" => { self.block_overlay_enabled = !self.block_overlay_enabled; self.block_overlay_enabled }
            "BreakProgress" => { self.break_progress_enabled = !self.break_progress_enabled; self.break_progress_enabled }
            "Chams" => { self.chams_enabled = !self.chams_enabled; self.chams_enabled }
            "ChestESP" => { self.chest_esp_enabled = !self.chest_esp_enabled; self.chest_esp_enabled }
            "DamageTags" => { self.damage_tags_enabled = !self.damage_tags_enabled; self.damage_tags_enabled }
            "DamageTint" => { self.damage_tint_enabled = !self.damage_tint_enabled; self.damage_tint_enabled }
            "ExtendCamera" => { self.extend_camera_enabled = !self.extend_camera_enabled; self.extend_camera_enabled }
            "FallView" => { self.fall_view_enabled = !self.fall_view_enabled; self.fall_view_enabled }
            "Freelook" => { self.freelook_enabled = !self.freelook_enabled; self.freelook_enabled }
            "Fullbright" => { self.fullbright_enabled = !self.fullbright_enabled; self.fullbright_enabled }
            "HUD" => { self.hud_enabled = !self.hud_enabled; self.hud_enabled }
            "HitParticles" => { self.hit_particles_enabled = !self.hit_particles_enabled; self.hit_particles_enabled }
            "Holdlook" => { self.holdlook_enabled = !self.holdlook_enabled; self.holdlook_enabled }
            "Indicators" => { self.indicators_enabled = !self.indicators_enabled; self.indicators_enabled }
            "ItemESP" => { self.item_esp_enabled = !self.item_esp_enabled; self.item_esp_enabled }
            "ItemPhysics" => { self.item_physics_enabled = !self.item_physics_enabled; self.item_physics_enabled }
            "Keystrokes" => { self.keystrokes_enabled = !self.keystrokes_enabled; self.keystrokes_enabled }
            "MobESP" => { self.mob_esp_enabled = !self.mob_esp_enabled; self.mob_esp_enabled }
            "Nametags" => { self.nametags_enabled = !self.nametags_enabled; self.nametags_enabled }
            "NoCameraClip" => { self.no_camera_clip_enabled = !self.no_camera_clip_enabled; self.no_camera_clip_enabled }
            "NoHurtCam" => { self.no_hurt_cam_enabled = !self.no_hurt_cam_enabled; self.no_hurt_cam_enabled }
            "PlayerESP" => { self.player_esp_enabled = !self.player_esp_enabled; self.player_esp_enabled }
            "PotionHUD" => { self.potion_hud_enabled = !self.potion_hud_enabled; self.potion_hud_enabled }
            "Radar" => { self.radar_enabled = !self.radar_enabled; self.radar_enabled }
            "Saturation" => { self.saturation_hud_enabled = !self.saturation_hud_enabled; self.saturation_hud_enabled }
            "TNTTimer" => { self.tnt_timer_enabled = !self.tnt_timer_enabled; self.tnt_timer_enabled }
            "TargetHUD" => { self.target_hud_enabled = !self.target_hud_enabled; self.target_hud_enabled }
            "Tracers" => { self.tracers_enabled = !self.tracers_enabled; self.tracers_enabled }
            "Trajectories" => { self.trajectories_enabled = !self.trajectories_enabled; self.trajectories_enabled }
            "Xray" => { self.xray_enabled = !self.xray_enabled; self.xray_enabled }

            "AutoRequeue" => { self.auto_requeue_enabled = !self.auto_requeue_enabled; self.auto_requeue_enabled }
            "AutoWho" => { self.auto_who_enabled = !self.auto_who_enabled; self.auto_who_enabled }
            "BedWars" => { self.bedwars_helper = !self.bedwars_helper; self.bedwars_helper }
            "BridgeInfo" => { self.bridge_info_enabled = !self.bridge_info_enabled; self.bridge_info_enabled }
            "DuelsStats" => { self.duels_stats_enabled = !self.duels_stats_enabled; self.duels_stats_enabled }
            "MurderMystery" => { self.murder_mystery_detector = !self.murder_mystery_detector; self.murder_mystery_detector }
            "SkyWars" => { self.skywars_helper = !self.skywars_helper; self.skywars_helper }
            "SpeedBuilders" => { self.speed_builders_helper = !self.speed_builders_helper; self.speed_builders_helper }
            "SumoFences" => { self.sumo_fences_enabled = !self.sumo_fences_enabled; self.sumo_fences_enabled }
            "WoolWars" => { self.wool_wars_helper = !self.wool_wars_helper; self.wool_wars_helper }

            "Anticheat" => { self.anticheat_module_enabled = !self.anticheat_module_enabled; self.anticheat_module_enabled }
            "ChatBypass" => { self.chat_bypass_enabled = !self.chat_bypass_enabled; self.chat_bypass_enabled }
            "FakeChat" => { self.fake_chat_enabled = !self.fake_chat_enabled; self.fake_chat_enabled }
            "LatencyAlerts" => { self.latency_alerts_enabled = !self.latency_alerts_enabled; self.latency_alerts_enabled }
            "NameHider" => { self.name_hider_enabled = !self.name_hider_enabled; self.name_hider_enabled }
            "ViewPackets" => { self.view_packets_enabled = !self.view_packets_enabled; self.view_packets_enabled }
            _ => false,
        };
        state
    }
}

/// Intercepts `.menu`, `.gui`, `.raven`, or any `.<module>` command typed in chat
pub fn handle_dot_command(msg: &str) -> Option<String> {
    let trimmed = msg.trim();
    if !trimmed.starts_with('.') {
        return None;
    }
    let cmd = trimmed[1..].trim();
    let mut raven = global_raven().lock().ok()?;

    if cmd.eq_ignore_ascii_case("menu")
        || cmd.eq_ignore_ascii_case("gui")
        || cmd.eq_ignore_ascii_case("raven")
    {
        raven.cfg.clickgui_open = !raven.cfg.clickgui_open;
        return Some(String::new());
    }

    // Match any module name directly (e.g. `.fastplace`, `.killaura`, `.scaffold`, `.sumo`)
    for cat in 0..7 {
        for &m in category_modules(cat) {
            if m.eq_ignore_ascii_case(cmd) {
                let state = raven.cfg.toggle(m);
                return Some(format!(
                    "[Raven bS+] {} -> {}",
                    m,
                    if state { "ON" } else { "OFF" }
                ));
            }
        }
    }

    Some(String::from(
        "[Raven bS+] Tape .menu pour ouvrir le ClickGUI Raven ou .<module> (ex: .fastplace, .scaffold, .killaura)",
    ))
}

/// Handles touch input for both the floating `[ R ]` button and the full 7-column Raven bS+ ClickGUI
pub fn handle_touch_event(
    x: f32,
    y: f32,
    screen_w: f32,
    screen_h: f32,
    started: bool,
    moved: bool,
    ended: bool,
) -> bool {
    let Ok(mut raven) = global_raven().lock() else {
        return false;
    };

    // 1. When ClickGUI is closed: only check the floating draggable `[ R ]` button
    if !raven.cfg.clickgui_open {
        let bx = raven.cfg.float_btn_x;
        let by = raven.cfg.float_btn_y;
        let bw = 28.0;
        let bh = 22.0;

        if started && x >= bx && x <= bx + bw && y >= by && y <= by + bh {
            raven.cfg.dragging_btn = true;
            return true;
        }
        if moved && raven.cfg.dragging_btn {
            raven.cfg.float_btn_x = (x - bw * 0.5).clamp(2.0, (screen_w - bw - 2.0).max(2.0));
            raven.cfg.float_btn_y = (y - bh * 0.5).clamp(2.0, (screen_h - bh - 2.0).max(2.0));
            return true;
        }
        if ended && raven.cfg.dragging_btn {
            raven.cfg.dragging_btn = false;
            if (x - (bx + bw * 0.5)).abs() < 14.0 && (y - (by + bh * 0.5)).abs() < 14.0 {
                raven.cfg.clickgui_open = true;
            }
            return true;
        }
        return false;
    }

    // 2. When ClickGUI is open: swallow all touches so the player never attacks/moves behind the menu
    if !started {
        return true;
    }

    // Check top-left `[ X ]` button (x: 6..30, y: 4..22) and top-right `[ X ]` button (x: screen_w-30..screen_w-6, y: 4..22)
    if (y >= 2.0 && y <= 24.0) && ((x >= 4.0 && x <= 32.0) || (x >= screen_w - 32.0 && x <= screen_w - 4.0)) {
        raven.cfg.clickgui_open = false;
        return true;
    }

    // Layout metrics matching `draw_hud_and_clickgui`
    let margin_x = 6.0;
    let gap = 3.0;
    let col_w = ((screen_w - margin_x * 2.0 - gap * 6.0) / 7.0).max(52.0);
    let top_y = 26.0;
    let header_h = 14.0;
    let row_h = 13.0;
    let settings_top = (screen_h - 64.0).max(top_y + header_h + row_h * 5.0);
    let max_rows = (((settings_top - top_y - header_h - 6.0) / row_h).floor() as usize).max(4);

    // Check clicks inside the 7 category columns
    for cat in 0..7 {
        let cx = margin_x + (cat as f32) * (col_w + gap);
        if x >= cx && x <= cx + col_w {
            // Header scroll buttons: left half scrolls up, right half scrolls down
            if y >= top_y && y <= top_y + header_h {
                let mods = category_modules(cat);
                if x < cx + col_w * 0.5 {
                    raven.cfg.col_scroll[cat] = raven.cfg.col_scroll[cat].saturating_sub(2);
                } else if raven.cfg.col_scroll[cat] + max_rows < mods.len() {
                    raven.cfg.col_scroll[cat] = (raven.cfg.col_scroll[cat] + 2).min(mods.len().saturating_sub(max_rows));
                }
                return true;
            }

            // Module rows
            let mods = category_modules(cat);
            let scroll = raven.cfg.col_scroll[cat];
            for (idx, &m) in mods.iter().skip(scroll).take(max_rows).enumerate() {
                let ry = top_y + header_h + (idx as f32) * row_h;
                if y >= ry && y <= ry + row_h {
                    // Rightmost 14px = `[+]` button to select this module for settings editor
                    if x >= cx + col_w - 14.0 {
                        raven.cfg.selected_module = m;
                    } else {
                        raven.cfg.toggle(m);
                        raven.cfg.selected_module = m;
                    }
                    return true;
                }
            }
        }
    }

    // Check clicks inside the bottom Settings Inspector Bar (y >= settings_top)
    if y >= settings_top && y <= screen_h - 4.0 {
        handle_settings_bar_touch(&mut raven.cfg, x, y, screen_w, settings_top);
    }

    true
}

fn handle_settings_bar_touch(cfg: &mut RavenConfig, x: f32, y: f32, screen_w: f32, top: f32) {
    let row1_y = top + 16.0;
    let row2_y = top + 34.0;
    let mid_x = screen_w * 0.5;

    let in_row1 = y >= row1_y && y <= row1_y + 15.0;
    let in_row2 = y >= row2_y && y <= row2_y + 15.0;
    let is_minus = (x >= 130.0 && x <= 165.0) || (x >= mid_x + 110.0 && x <= mid_x + 145.0);
    let is_plus = (x >= 170.0 && x <= 205.0) || (x >= mid_x + 150.0 && x <= mid_x + 185.0);
    let delta: f32 = if is_minus { -1.0 } else if is_plus { 1.0 } else { 0.0 };

    match cfg.selected_module {
        "FastPlace" => {
            if in_row1 && delta != 0.0 {
                cfg.fast_place_delay = (cfg.fast_place_delay as i32 + delta as i32).clamp(0, 4) as u8;
            }
        }
        "Reach" => {
            if in_row1 && delta != 0.0 {
                cfg.reach_min = (cfg.reach_min + f64::from(delta) * 0.1).clamp(3.0, 6.0);
                if cfg.reach_max < cfg.reach_min {
                    cfg.reach_max = cfg.reach_min;
                }
            } else if in_row2 && delta != 0.0 {
                cfg.reach_max = (cfg.reach_max + f64::from(delta) * 0.1).clamp(cfg.reach_min, 6.0);
            }
        }
        "KillAura" => {
            if in_row1 && x < mid_x && delta != 0.0 {
                cfg.killaura_range = (cfg.killaura_range + f64::from(delta) * 0.1).clamp(3.0, 6.0);
            } else if in_row1 && x >= mid_x {
                cfg.killaura_mode = cfg.killaura_mode.next();
            } else if in_row2 && delta != 0.0 {
                cfg.killaura_max_cps = (cfg.killaura_max_cps + delta).clamp(4.0, 25.0);
            }
        }
        "Velocity" => {
            if in_row1 && delta != 0.0 {
                cfg.velocity_horizontal = (cfg.velocity_horizontal + delta * 5.0).clamp(0.0, 100.0);
            } else if in_row2 && delta != 0.0 {
                cfg.velocity_vertical = (cfg.velocity_vertical + delta * 5.0).clamp(0.0, 100.0);
            }
        }
        "Scaffold" => {
            if in_row1 && x < mid_x {
                cfg.scaffold_sprint = cfg.scaffold_sprint.next();
            } else if in_row1 && x >= mid_x {
                cfg.scaffold_schedule = cfg.scaffold_schedule.next();
            } else if in_row2 && x < mid_x {
                cfg.scaffold_rotation = cfg.scaffold_rotation.next();
            } else if in_row2 && x >= mid_x {
                cfg.scaffold_tower = !cfg.scaffold_tower;
            }
        }
        "Fly" => {
            if in_row1 && x < mid_x {
                cfg.fly_mode = cfg.fly_mode.next();
            } else if in_row2 && delta != 0.0 {
                cfg.fly_speed = (cfg.fly_speed + f64::from(delta) * 0.2).clamp(0.2, 5.0);
            }
        }
        "Speed" => {
            if in_row1 && x < mid_x {
                cfg.speed_mode = cfg.speed_mode.next();
            } else if in_row2 && delta != 0.0 {
                cfg.speed_multiplier = (cfg.speed_multiplier + f64::from(delta) * 0.05).clamp(1.0, 3.0);
            }
        }
        "Timer" => {
            if in_row1 && delta != 0.0 {
                cfg.timer_value = (cfg.timer_value + delta * 0.05).clamp(0.5, 5.0);
            }
        }
        "AutoClicker" => {
            if in_row1 && delta != 0.0 {
                cfg.autoclicker_min_cps = (cfg.autoclicker_min_cps + delta).clamp(1.0, 25.0);
            } else if in_row2 && delta != 0.0 {
                cfg.autoclicker_max_cps = (cfg.autoclicker_max_cps + delta).clamp(cfg.autoclicker_min_cps, 30.0);
            }
        }
        "HitBox" => {
            if in_row1 && delta != 0.0 {
                cfg.hitbox_expand = (cfg.hitbox_expand + f64::from(delta) * 0.05).clamp(0.0, 1.0);
            }
        }
        "BackTrack" => {
            if in_row1 && delta != 0.0 {
                cfg.backtrack_delay_ms = (cfg.backtrack_delay_ms as i64 + (delta as i64) * 25).clamp(25, 600) as u64;
            }
        }
        "SumoFences" => {
            if in_row1 && delta != 0.0 {
                cfg.sumo_fences_height = (cfg.sumo_fences_height + delta * 0.5).clamp(1.0, 4.0);
            }
        }
        _ => {
            if in_row1 {
                cfg.toggle(cfg.selected_module);
            }
        }
    }
}

/// Draws the in-game Raven HUD, floating `[ R ]` button, and full 7-column Raven bS+ ClickGUI
pub fn draw_hud_and_clickgui(batch: &mut HudBatch, atlas: &HudAtlas) {
    let Ok(raven) = global_raven().lock() else {
        return;
    };
    let cfg = &raven.cfg;

    // 1. In-game HUD ArrayList & Floating `[ R ]` button when ClickGUI is closed
    if !cfg.clickgui_open {
        if cfg.hud_enabled {
            batch.text(
                atlas,
                "Raven bS+ v1.3.3",
                6.0,
                6.0,
                rgba(168, 85, 247, 255),
                true,
            );
            let mut ay = 18.0;
            for cat in 0..7 {
                for &m in category_modules(cat) {
                    if m != "HUD" && m != "Keystrokes" && cfg.is_enabled(m) {
                        let tw = HudBatch::text_width(atlas, m);
                        batch.fill(
                            atlas,
                            batch.width - tw - 8.0,
                            ay - 1.0,
                            tw + 6.0,
                            11.0,
                            rgba(15, 15, 22, 165),
                        );
                        batch.text(
                            atlas,
                            m,
                            batch.width - tw - 5.0,
                            ay,
                            rgba(46, 204, 113, 255),
                            true,
                        );
                        ay += 11.0;
                    }
                }
            }
        }

        // Draggable Floating `[ R ]` button
        let (bx, by) = (cfg.float_btn_x, cfg.float_btn_y);
        batch.fill(atlas, bx, by, 28.0, 22.0, rgba(20, 20, 30, 215));
        batch.fill(atlas, bx, by, 28.0, 2.0, rgba(168, 85, 247, 255));
        batch.text(atlas, "[R]", bx + 5.0, by + 7.0, rgba(255, 255, 255, 255), true);
        return;
    }

    // 2. Full Raven bS+ ClickGUI Overlay
    let sw = batch.width;
    let sh = batch.height;
    batch.fill_gradient(
        atlas,
        (0.0, 0.0, sw, sh),
        rgba(10, 10, 16, 225),
        rgba(18, 14, 28, 235),
    );

    // Top Bar + Left & Right `[ X ]` Close Buttons
    batch.fill(atlas, 0.0, 0.0, sw, 22.0, rgba(18, 18, 26, 245));
    batch.fill(atlas, 0.0, 21.0, sw, 1.0, rgba(168, 85, 247, 255));

    // Top-Left `[ X ]` Close Button
    batch.fill(atlas, 4.0, 3.0, 26.0, 16.0, rgba(231, 76, 60, 240));
    batch.text(atlas, "X", 14.0, 7.0, rgba(255, 255, 255, 255), true);

    // Top-Right `[ X ]` Close Button
    batch.fill(atlas, sw - 30.0, 3.0, 26.0, 16.0, rgba(231, 76, 60, 240));
    batch.text(atlas, "X", sw - 20.0, 7.0, rgba(255, 255, 255, 255), true);

    // Center Title
    let title = "Raven bS+ ClickGUI (Tap Module = ON/OFF | [+] = Settings | [X] = Close)";
    let tw = HudBatch::text_width(atlas, title);
    batch.text(
        atlas,
        title,
        ((sw - tw) * 0.5).max(36.0),
        7.0,
        rgba(220, 190, 255, 255),
        true,
    );

    // 7 Category Columns
    let margin_x = 6.0;
    let gap = 3.0;
    let col_w = ((sw - margin_x * 2.0 - gap * 6.0) / 7.0).max(52.0);
    let top_y = 26.0;
    let header_h = 14.0;
    let row_h = 13.0;
    let settings_top = (sh - 64.0).max(top_y + header_h + row_h * 5.0);
    let max_rows = (((settings_top - top_y - header_h - 6.0) / row_h).floor() as usize).max(4);

    for (cat, &cat_name) in CATEGORIES.iter().enumerate() {
        let cx = margin_x + (cat as f32) * (col_w + gap);
        let mods = category_modules(cat);
        let scroll = cfg.col_scroll[cat];
        let visible_count = mods.len().saturating_sub(scroll).min(max_rows);
        let col_h = header_h + (visible_count as f32) * row_h + 2.0;

        // Column background & Raven purple header
        batch.fill(atlas, cx, top_y, col_w, col_h, rgba(16, 16, 24, 230));
        batch.fill(atlas, cx, top_y, col_w, header_h, rgba(142, 68, 173, 255));
        batch.text(
            atlas,
            cat_name,
            cx + 3.0,
            top_y + 3.0,
            rgba(255, 255, 255, 255),
            true,
        );
        if mods.len() > max_rows {
            batch.text(
                atlas,
                "<>",
                cx + col_w - 14.0,
                top_y + 3.0,
                rgba(230, 230, 250, 255),
                false,
            );
        }

        // Module rows
        for (idx, &m) in mods.iter().skip(scroll).take(max_rows).enumerate() {
            let ry = top_y + header_h + (idx as f32) * row_h;
            let on = cfg.is_enabled(m);
            let selected = cfg.selected_module == m;

            let bg = if on {
                rgba(125, 60, 190, 230)
            } else if selected {
                rgba(42, 42, 60, 230)
            } else {
                rgba(24, 24, 34, 215)
            };
            batch.fill(atlas, cx + 1.0, ry, col_w - 2.0, row_h - 1.0, bg);

            let txt_col = if on {
                rgba(255, 255, 255, 255)
            } else {
                rgba(185, 185, 200, 255)
            };
            batch.text(atlas, m, cx + 3.0, ry + 2.0, txt_col, on);
            batch.text(
                atlas,
                "+",
                cx + col_w - 9.0,
                ry + 2.0,
                if selected {
                    rgba(46, 204, 113, 255)
                } else {
                    rgba(150, 150, 170, 255)
                },
                false,
            );
        }
    }

    // Bottom Settings Inspector Panel for `cfg.selected_module`
    batch.fill(
        atlas,
        margin_x,
        settings_top,
        sw - margin_x * 2.0,
        sh - settings_top - 4.0,
        rgba(18, 18, 28, 245),
    );
    batch.fill(
        atlas,
        margin_x,
        settings_top,
        sw - margin_x * 2.0,
        2.0,
        rgba(46, 204, 113, 255),
    );

    let sel = cfg.selected_module;
    let status = if cfg.is_enabled(sel) { "ON" } else { "OFF" };
    batch.text(
        atlas,
        &format!("Settings: {} [{}]", sel, status),
        margin_x + 6.0,
        settings_top + 4.0,
        rgba(46, 204, 113, 255),
        true,
    );

    draw_module_settings_rows(batch, atlas, cfg, margin_x + 6.0, settings_top, sw);
}

fn draw_slider_row(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    x: f32,
    y: f32,
    label: &str,
    val_str: &str,
) {
    batch.text(
        atlas,
        &format!("{}: {}", label, val_str),
        x,
        y + 3.0,
        rgba(235, 235, 245, 255),
        true,
    );
    // `[-]` button
    batch.fill(atlas, x + 124.0, y, 32.0, 14.0, rgba(192, 57, 43, 235));
    batch.text(atlas, " - ", x + 131.0, y + 3.0, rgba(255, 255, 255, 255), true);
    // `[+]` button
    batch.fill(atlas, x + 162.0, y, 32.0, 14.0, rgba(39, 174, 96, 235));
    batch.text(atlas, " + ", x + 169.0, y + 3.0, rgba(255, 255, 255, 255), true);
}

fn draw_mode_row(batch: &mut HudBatch, atlas: &HudAtlas, x: f32, y: f32, label: &str, mode: &str) {
    batch.fill(atlas, x, y, 185.0, 14.0, rgba(52, 73, 94, 235));
    batch.text(
        atlas,
        &format!("{}: [{}] (Tap)", label, mode),
        x + 4.0,
        y + 3.0,
        rgba(255, 255, 255, 255),
        true,
    );
}

fn draw_module_settings_rows(
    batch: &mut HudBatch,
    atlas: &HudAtlas,
    cfg: &RavenConfig,
    x: f32,
    top: f32,
    sw: f32,
) {
    let r1 = top + 16.0;
    let r2 = top + 34.0;
    let mid = sw * 0.5 + 4.0;

    match cfg.selected_module {
        "FastPlace" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Tick Delay",
                &format!("{} ticks", cfg.fast_place_delay),
            );
            batch.text(
                atlas,
                "Blocks Only: ON | Pitch Check: OFF",
                x,
                r2 + 3.0,
                rgba(170, 170, 190, 255),
                false,
            );
        }
        "Reach" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Reach Min",
                &format!("{:.1}m", cfg.reach_min),
            );
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Reach Max",
                &format!("{:.1}m", cfg.reach_max),
            );
        }
        "KillAura" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Range",
                &format!("{:.1}m", cfg.killaura_range),
            );
            draw_mode_row(batch, atlas, mid, r1, "Mode", cfg.killaura_mode.as_str());
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Max CPS",
                &format!("{:.0}", cfg.killaura_max_cps),
            );
        }
        "Velocity" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Horizontal",
                &format!("{:.0}%", cfg.velocity_horizontal),
            );
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Vertical",
                &format!("{:.0}%", cfg.velocity_vertical),
            );
        }
        "Scaffold" => {
            draw_mode_row(
                batch,
                atlas,
                x,
                r1,
                "Sprint",
                cfg.scaffold_sprint.as_str(),
            );
            draw_mode_row(
                batch,
                atlas,
                mid,
                r1,
                "Schedule",
                cfg.scaffold_schedule.as_str(),
            );
            draw_mode_row(
                batch,
                atlas,
                x,
                r2,
                "Rotation",
                cfg.scaffold_rotation.as_str(),
            );
            draw_mode_row(
                batch,
                atlas,
                mid,
                r2,
                "Tower",
                if cfg.scaffold_tower { "ON" } else { "OFF" },
            );
        }
        "Fly" => {
            draw_mode_row(batch, atlas, x, r1, "Fly Mode", cfg.fly_mode.as_str());
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Fly Speed",
                &format!("{:.1}x", cfg.fly_speed),
            );
        }
        "Speed" => {
            draw_mode_row(batch, atlas, x, r1, "Speed Mode", cfg.speed_mode.as_str());
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Multiplier",
                &format!("{:.2}x", cfg.speed_multiplier),
            );
        }
        "Timer" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Game Speed",
                &format!("{:.2}x", cfg.timer_value),
            );
        }
        "AutoClicker" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Min CPS",
                &format!("{:.0}", cfg.autoclicker_min_cps),
            );
            draw_slider_row(
                batch,
                atlas,
                x,
                r2,
                "Max CPS",
                &format!("{:.0}", cfg.autoclicker_max_cps),
            );
        }
        "HitBox" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Expand",
                &format!("+{:.2}m", cfg.hitbox_expand),
            );
        }
        "BackTrack" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Delay",
                &format!("{} ms", cfg.backtrack_delay_ms),
            );
        }
        "SumoFences" => {
            draw_slider_row(
                batch,
                atlas,
                x,
                r1,
                "Fence Height",
                &format!("{:.1} blocks", cfg.sumo_fences_height),
            );
        }
        _ => {
            draw_mode_row(
                batch,
                atlas,
                x,
                r1,
                "State",
                if cfg.is_enabled(cfg.selected_module) {
                    "ENABLED"
                } else {
                    "DISABLED"
                },
            );
        }
    }
}
