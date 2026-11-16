// File: crates/horizon-client/src/raven_bs.rs
//! Complete Raven bS+ (1.3.3+1) Native Rust Engine + Touch ClickGUI + Full `Game` & `Remote` Wiring
//! Wired for both Singleplayer (integrated server over `self.remote`) and Multiplayer/LAN on iPadOS.

#![allow(dead_code, clippy::too_many_arguments, clippy::type_complexity)]

use crate::hud::{HudAtlas, HudBatch, rgba};
use std::collections::VecDeque;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

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
    // COMBAT
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

    // MOVEMENT
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

    // PLAYER
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

    // WORLD
    pub antibot_enabled: bool,
    pub scaffold_enabled: bool,
    pub scaffold_sprint: ScaffoldSprintMode,
    pub scaffold_rotation: ScaffoldRotationMode,
    pub scaffold_schedule: ScaffoldScheduleMode,
    pub scaffold_tower: bool,
    pub weather_clear: bool,

    // RENDER
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

    // MINIGAMES & OTHER
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

    // CLICKGUI STATE
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
            aim_assist_speed: 55.0,
            aim_assist_fov: 90.0,
            aim_assist_distance: 4.5,
            autoclicker_enabled: false,
            autoclicker_min_cps: 12.0,
            autoclicker_max_cps: 16.0,
            autoclicker_jitter: 0.25,
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
            hitbox_expand: 0.35,
            hit_select_enabled: false,
            jump_reset_enabled: false,
            killaura_enabled: false,
            killaura_mode: KillAuraMode::Switch,
            killaura_range: 4.2,
            killaura_fov: 360.0,
            killaura_min_cps: 12.0,
            killaura_max_cps: 16.0,
            killaura_rot_speed: 85.0,
            knockback_delay_enabled: false,
            knockback_delay_ms: 90,
            lag_range_enabled: false,
            lag_range_ticks: 4,
            piercing_enabled: false,
            reach_enabled: false,
            reach_min: 3.5,
            reach_max: 4.5,
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
            speed_multiplier: 1.45,
            sprint_enabled: true,
            sprint_omni: false,
            stasis_enabled: false,
            step_enabled: false,
            step_height: 1.5,
            stop_motion_enabled: false,
            teleport_enabled: false,
            timer_enabled: false,
            timer_value: 1.35,
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
            fast_mine_multiplier: 1.6,
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

            antibot_enabled: true,
            scaffold_enabled: false,
            scaffold_sprint: ScaffoldSprintMode::Vanilla,
            scaffold_rotation: ScaffoldRotationMode::Precise,
            scaffold_schedule: ScaffoldScheduleMode::Normal,
            scaffold_tower: true,
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
            hud_enabled: true,
            hit_particles_enabled: false,
            holdlook_enabled: false,
            indicators_enabled: false,
            item_esp_enabled: false,
            item_physics_enabled: false,
            keystrokes_enabled: true,
            mob_esp_enabled: false,
            nametags_enabled: false,
            no_camera_clip_enabled: false,
            no_hurt_cam_enabled: false,
            player_esp_enabled: false,
            potion_hud_enabled: false,
            radar_enabled: false,
            saturation_hud_enabled: false,
            tnt_timer_enabled: false,
            target_hud_enabled: true,
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
            selected_module: "Scaffold",
            col_scroll: [0; 7],
        }
    }
}

pub struct RavenEngine {
    pub cfg: RavenConfig,
    pub last_attack: Instant,
    pub lmb_clicks: VecDeque<Instant>,
    pub rmb_clicks: VecDeque<Instant>,
    pub switch_idx: usize,
    pub air_ticks: u32,
    pub freecam_anchor: Option<[f64; 3]>,
    pub freelook_anchor: Option<(f32, f32)>,
    pub target_info: Option<(String, f32, f32, f64)>,
    pub blocks_placed_session: u32,
    pub prng: u64,
    pub timer_was_active: bool,
    pub fullbright_was_active: bool,
    pub fastmine_was_active: bool,
}

impl Default for RavenEngine {
    fn default() -> Self {
        Self {
            cfg: RavenConfig::default(),
            last_attack: Instant::now(),
            lmb_clicks: VecDeque::new(),
            rmb_clicks: VecDeque::new(),
            switch_idx: 0,
            air_ticks: 0,
            freecam_anchor: None,
            freelook_anchor: None,
            target_info: None,
            blocks_placed_session: 0,
            prng: 0x9E37_79B9_7F4A_7C15,
            timer_was_active: false,
            fullbright_was_active: false,
            fastmine_was_active: false,
        }
    }
}

impl RavenEngine {
    #[inline]
    pub fn rand_f32(&mut self) -> f32 {
        let mut x = self.prng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.prng = x;
        let r = x.wrapping_mul(0x2545_F491_4F6C_DD1D);
        ((r >> 40) as u32) as f32 / ((1u32 << 24) as f32)
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

pub fn category_modules(cat: usize) -> &'static [&'static str] {
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
        match name {
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
        }
    }
}

#[inline]
pub fn wrap_deg(mut deg: f32) -> f32 {
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
pub fn calc_angles(eye: [f64; 3], target: [f64; 3]) -> (f32, f32) {
    let dx = target[0] - eye[0];
    let dy = target[1] - eye[1];
    let dz = target[2] - eye[2];
    let dist = (dx * dx + dz * dz).sqrt();
    let yaw = (dz.atan2(dx).to_degrees() as f32) - 90.0;
    let pitch = -(dy.atan2(dist).to_degrees() as f32);
    (wrap_deg(yaw), pitch.clamp(-90.0, 90.0))
}

#[inline]
pub fn smooth_rot(cur: (f32, f32), target: (f32, f32), speed: f32) -> (f32, f32) {
    let dyaw = wrap_deg(target.0 - cur.0);
    let dpitch = target.1 - cur.1;
    let factor = (speed / 100.0).clamp(0.05, 1.0);
    (
        wrap_deg(cur.0 + dyaw * factor),
        (cur.1 + dpitch * factor).clamp(-90.0, 90.0),
    )
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
        "[Raven bS+] Tape .menu pour ouvrir le ClickGUI Raven ou .<module> (ex: .fly, .killaura, .scaffold, .speed)",
    ))
}

/// Handles touch input for both the floating `[ R ]` button and the 7-column Raven bS+ ClickGUI
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
            if (x - (bx + bw * 0.5)).abs() < 16.0 && (y - (by + bh * 0.5)).abs() < 16.0 {
                raven.cfg.clickgui_open = true;
            }
            return true;
        }
        return false;
    }

    if !started {
        return true;
    }

    if (y >= 2.0 && y <= 24.0)
        && ((x >= 4.0 && x <= 32.0) || (x >= screen_w - 32.0 && x <= screen_w - 4.0))
    {
        raven.cfg.clickgui_open = false;
        return true;
    }

    let margin_x = 6.0;
    let gap = 3.0;
    let col_w = ((screen_w - margin_x * 2.0 - gap * 6.0) / 7.0).max(52.0);
    let top_y = 26.0;
    let header_h = 14.0;
    let row_h = 13.0;
    let settings_top = (screen_h - 64.0).max(top_y + header_h + row_h * 5.0);
    let max_rows = (((settings_top - top_y - header_h - 6.0) / row_h).floor() as usize).max(4);

    for cat in 0..7 {
        let cx = margin_x + (cat as f32) * (col_w + gap);
        if x >= cx && x <= cx + col_w {
            if y >= top_y && y <= top_y + header_h {
                let mods = category_modules(cat);
                if x < cx + col_w * 0.5 {
                    raven.cfg.col_scroll[cat] = raven.cfg.col_scroll[cat].saturating_sub(2);
                } else if raven.cfg.col_scroll[cat] + max_rows < mods.len() {
                    raven.cfg.col_scroll[cat] =
                        (raven.cfg.col_scroll[cat] + 2).min(mods.len().saturating_sub(max_rows));
                }
                return true;
            }

            let mods = category_modules(cat);
            let scroll = raven.cfg.col_scroll[cat];
            for (idx, &m) in mods.iter().skip(scroll).take(max_rows).enumerate() {
                let ry = top_y + header_h + (idx as f32) * row_h;
                if y >= ry && y <= ry + row_h {
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
    let delta: f32 = if is_minus {
        -1.0
    } else if is_plus {
        1.0
    } else {
        0.0
    };

    match cfg.selected_module {
        "FastPlace" => {
            if in_row1 && delta != 0.0 {
                cfg.fast_place_delay =
                    (cfg.fast_place_delay as i32 + delta as i32).clamp(0, 4) as u8;
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
                cfg.killaura_range = (cfg.killaura_range + f64::from(delta) * 0.2).clamp(3.0, 6.5);
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
                cfg.speed_multiplier =
                    (cfg.speed_multiplier + f64::from(delta) * 0.05).clamp(1.0, 3.0);
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
                cfg.autoclicker_max_cps =
                    (cfg.autoclicker_max_cps + delta).clamp(cfg.autoclicker_min_cps, 30.0);
            }
        }
        "HitBox" => {
            if in_row1 && delta != 0.0 {
                cfg.hitbox_expand = (cfg.hitbox_expand + f64::from(delta) * 0.05).clamp(0.0, 1.5);
            }
        }
        "BackTrack" => {
            if in_row1 && delta != 0.0 {
                cfg.backtrack_delay_ms =
                    (cfg.backtrack_delay_ms as i64 + (delta as i64) * 25).clamp(25, 600) as u64;
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

pub fn draw_hud_and_clickgui(batch: &mut HudBatch, atlas: &HudAtlas) {
    let Ok(raven) = global_raven().lock() else {
        return;
    };
    let cfg = &raven.cfg;

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

        if cfg.target_hud_enabled {
            if let Some((ref tname, hp, max_hp, dist)) = raven.target_info {
                let tx = (batch.width * 0.5 + 24.0).min(batch.width - 130.0);
                let ty = batch.height * 0.5 + 16.0;
                batch.fill(atlas, tx, ty, 122.0, 34.0, rgba(16, 16, 26, 215));
                batch.fill(atlas, tx, ty, 122.0, 2.0, rgba(168, 85, 247, 255));
                batch.text(atlas, tname, tx + 4.0, ty + 4.0, rgba(255, 255, 255, 255), true);
                batch.text(
                    atlas,
                    &format!("{:.1} HP | {:.1}m", hp, dist),
                    tx + 4.0,
                    ty + 14.0,
                    rgba(46, 204, 113, 255),
                    true,
                );
                let ratio = (hp / max_hp.max(1.0)).clamp(0.0, 1.0);
                batch.fill(atlas, tx + 4.0, ty + 25.0, 114.0, 5.0, rgba(45, 45, 60, 255));
                batch.fill(
                    atlas,
                    tx + 4.0,
                    ty + 25.0,
                    114.0 * ratio,
                    5.0,
                    rgba(46, 204, 113, 255),
                );
            }
        }

        let (bx, by) = (cfg.float_btn_x, cfg.float_btn_y);
        batch.fill(atlas, bx, by, 28.0, 22.0, rgba(20, 20, 30, 215));
        batch.fill(atlas, bx, by, 28.0, 2.0, rgba(168, 85, 247, 255));
        batch.text(atlas, "[R]", bx + 5.0, by + 7.0, rgba(255, 255, 255, 255), true);
        return;
    }

    let sw = batch.width;
    let sh = batch.height;
    batch.fill_gradient(
        atlas,
        (0.0, 0.0, sw, sh),
        rgba(10, 10, 16, 225),
        rgba(18, 14, 28, 235),
    );

    batch.fill(atlas, 0.0, 0.0, sw, 22.0, rgba(18, 18, 26, 245));
    batch.fill(atlas, 0.0, 21.0, sw, 1.0, rgba(168, 85, 247, 255));

    batch.fill(atlas, 4.0, 3.0, 26.0, 16.0, rgba(231, 76, 60, 240));
    batch.text(atlas, "X", 14.0, 7.0, rgba(255, 255, 255, 255), true);
    batch.fill(atlas, sw - 30.0, 3.0, 26.0, 16.0, rgba(231, 76, 60, 240));
    batch.text(atlas, "X", sw - 20.0, 7.0, rgba(255, 255, 255, 255), true);

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
    batch.fill(atlas, x + 124.0, y, 32.0, 14.0, rgba(192, 57, 43, 235));
    batch.text(atlas, " - ", x + 131.0, y + 3.0, rgba(255, 255, 255, 255), true);
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
                "Hold Finger = Auto Place 20 blocks/s",
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

// ============================================================================
// REAL GAMEPLAY HOOKS MACRO FOR `crates/horizon-client/src/game.rs` (V4 - FULL REMOTE + LOCAL PACKET SYNC)
// ============================================================================

#[macro_export]
macro_rules! impl_raven_game {
    () => {
        impl Game<'_> {
            /// Finds a solid neighbor around `target` `[x, y, z]` and returns `Hit` to place a block at `target`.
            fn raven_find_place_hit(&self, target: [i32; 3]) -> Option<Hit> {
                if !self
                    .res
                    .facts()
                    .is_air(self.world.block(target[0], target[1], target[2]))
                {
                    return None;
                }
                let dirs: [[i32; 3]; 6] = [
                    [0, -1, 0],
                    [0, 1, 0],
                    [0, 0, -1],
                    [0, 0, 1],
                    [-1, 0, 0],
                    [1, 0, 0],
                ];
                for d in dirs {
                    let nb = [target[0] + d[0], target[1] + d[1], target[2] + d[2]];
                    let st = self.world.block(nb[0], nb[1], nb[2]);
                    let facts = self.res.facts().get(st);
                    if !self.res.facts().is_air(st) && facts.fluid == Fluid::None {
                        let face = [-d[0], -d[1], -d[2]];
                        let point = [
                            f64::from(nb[0]) + 0.5 + f64::from(face[0]) * 0.5,
                            f64::from(nb[1]) + 0.5 + f64::from(face[1]) * 0.5,
                            f64::from(nb[2]) + 0.5 + f64::from(face[2]) * 0.5,
                        ];
                        return Some(Hit {
                            pos: nb,
                            face,
                            point,
                            inside: false,
                        });
                    }
                }
                // Extended 2-block search for diagonal/sprint bridging
                for d1 in dirs {
                    let mid = [target[0] + d1[0], target[1] + d1[1], target[2] + d1[2]];
                    if !self
                        .res
                        .facts()
                        .is_air(self.world.block(mid[0], mid[1], mid[2]))
                    {
                        continue;
                    }
                    for d2 in dirs {
                        let nb = [mid[0] + d2[0], mid[1] + d2[1], mid[2] + d2[2]];
                        let st = self.world.block(nb[0], nb[1], nb[2]);
                        let facts = self.res.facts().get(st);
                        if !self.res.facts().is_air(st) && facts.fluid == Fluid::None {
                            let face = [-d2[0], -d2[1], -d2[2]];
                            let point = [
                                f64::from(nb[0]) + 0.5 + f64::from(face[0]) * 0.5,
                                f64::from(nb[1]) + 0.5 + f64::from(face[1]) * 0.5,
                                f64::from(nb[2]) + 0.5 + f64::from(face[2]) * 0.5,
                            ];
                            return Some(Hit {
                                pos: nb,
                                face,
                                point,
                                inside: false,
                            });
                        }
                    }
                }
                None
            }

            /// Directly places a block at `target_cell` both in the local client world AND sends `S::UseItemOn` + `S::Swing` to the server!
            fn raven_place_block_at(&mut self, target_cell: [i32; 3], hit: Hit) -> bool {
                let Some(stack) = self.inventory.get(self.selected).and_then(|s| s.clone()) else {
                    return false;
                };
                if stack.count == 0 {
                    return false;
                }
                let block_state = self
                    .pack
                    .assets
                    .registry
                    .default_state(&stack.item)
                    .or_else(|| {
                        self.items()
                            .get(&stack.item)
                            .and_then(|info| info.block.as_deref())
                            .and_then(|bname| self.pack.assets.registry.default_state(bname))
                    });
                let Some(state) = block_state else {
                    return false;
                };
                if self.res.facts().is_air(state) {
                    return false;
                }

                // 1. Update client world immediately so player stands on the block right this frame
                let _ = self.world.set(target_cell[0], target_cell[1], target_cell[2], state);
                let _ = self.level_set(target_cell, state);

                // 2. If connected to integrated/remote server, send real UseItemOn + Swing packets
                if self.remote.is_some() {
                    let face_id: i32 = match hit.face {
                        [0, -1, 0] => 0,
                        [0, 1, 0] => 1,
                        [0, 0, -1] => 2,
                        [0, 0, 1] => 3,
                        [-1, 0, 0] => 4,
                        [1, 0, 0] => 5,
                        _ => 1,
                    };
                    let cursor = [
                        (hit.point[0] - f64::from(hit.pos[0])).clamp(0.0, 1.0) as f32,
                        (hit.point[1] - f64::from(hit.pos[1])).clamp(0.0, 1.0) as f32,
                        (hit.point[2] - f64::from(hit.pos[2])).clamp(0.0, 1.0) as f32,
                    ];
                    let seq = if let Some(r) = self.remote.as_mut() {
                        r.sequence += 1;
                        r.sequence
                    } else {
                        1
                    };
                    self.send(horizon_protocol::packets::play::Serverbound::UseItemOn(
                        horizon_protocol::packets::play::UseItemOn {
                            hand: 0,
                            pos: hit.pos,
                            face: face_id,
                            cursor,
                            inside: hit.inside,
                            world_border: false,
                            sequence: seq,
                        },
                    ));
                    self.send(horizon_protocol::packets::play::Serverbound::Swing(
                        horizon_protocol::packets::play::Swing { hand: 0 },
                    ));
                }

                self.swing_arm();
                if !self.creative {
                    if let Some(Some(st)) = self.inventory.get_mut(self.selected) {
                        st.count = st.count.saturating_sub(1);
                        if st.count == 0 {
                            self.inventory[self.selected] = None;
                        }
                    }
                }
                true
            }

            /// Directly executes all active Raven bS+ modules on `Game` & `Remote` without getting overwritten
            pub(crate) fn raven_tick(&mut self) {
                self.forced_hit = None;

                // Read D-Pad (`self.keys.held`), Touch Stick, and Keyboard
                let stick = self.touch_move().unwrap_or([0.0, 0.0]);
                let mut move_fwd = stick[1];
                let mut move_left = stick[0];
                if self.keys.held(Mapping::Forward)
                    || self.physical_keys.contains(&KeyCode::KeyW)
                {
                    move_fwd += 1.0;
                }
                if self.keys.held(Mapping::Back)
                    || self.physical_keys.contains(&KeyCode::KeyS)
                {
                    move_fwd -= 1.0;
                }
                if self.keys.held(Mapping::Left)
                    || self.physical_keys.contains(&KeyCode::KeyA)
                {
                    move_left += 1.0;
                }
                if self.keys.held(Mapping::Right)
                    || self.physical_keys.contains(&KeyCode::KeyD)
                {
                    move_left -= 1.0;
                }
                let is_moving = move_fwd.abs() > 0.05 || move_left.abs() > 0.05;
                let jump_held = self.keys.held(Mapping::Jump)
                    || self.physical_keys.contains(&KeyCode::Space);
                let sneak_held = self.keys.held(Mapping::Sneak)
                    || self.physical_keys.contains(&KeyCode::ShiftLeft);

                let mut move_yaw = self.yaw;
                if move_fwd < -0.05 {
                    move_yaw += 180.0;
                }
                let diag = if move_fwd < -0.05 {
                    -0.5
                } else if move_fwd > 0.05 {
                    0.5
                } else {
                    1.0
                };
                if move_left > 0.05 {
                    move_yaw -= 90.0 * diag;
                } else if move_left < -0.05 {
                    move_yaw += 90.0 * diag;
                }
                let dir_rad = f64::from(move_yaw).to_radians();
                let dir_x = -dir_rad.sin();
                let dir_z = dir_rad.cos();

                let (p_pos, on_ground, horiz_col, in_liquid) =
                    if let Some(p) = self.player.as_ref() {
                        let bx = p.position.x.floor() as i32;
                        let bz = p.position.z.floor() as i32;
                        let feet_st = self.world.block(bx, p.position.y.floor() as i32, bz);
                        let liq = self.res.facts().get(feet_st).fluid != Fluid::None;
                        (
                            [p.position.x, p.position.y, p.position.z],
                            p.ground.on_ground,
                            p.collision.horizontal,
                            liq,
                        )
                    } else {
                        return;
                    };

                // Extract config snapshot
                let (
                    timer_en,
                    timer_val,
                    fullbright_en,
                    anti_debuff_en,
                    fast_mine_en,
                    reach_en,
                    reach_max,
                    hitbox_en,
                    hitbox_exp,
                    delay_rem_en,
                    fast_place_en,
                    fast_place_del,
                    sprint_en,
                    nofall_en,
                    noslow_en,
                    auto_jump_en,
                    jump_reset_en,
                    vel_en,
                    vel_h,
                    vel_v,
                    no_hurt_en,
                    vclip_en,
                    vclip_dist,
                    stop_mot_en,
                    stasis_en,
                    fly_en,
                    freecam_en,
                    fly_mode,
                    fly_spd,
                    freecam_spd,
                    speed_en,
                    bhop_en,
                    boost_en,
                    speed_mode,
                    speed_mult,
                    long_jump_en,
                    long_jump_boost,
                    inst_stop_en,
                    step_en,
                    step_h,
                    dolphin_en,
                    safewalk_en,
                    bridge_assist_en,
                    ka_en,
                    ka_range,
                    ka_min_cps,
                    ka_max_cps,
                    autoblock_en,
                    auto_swap_en,
                    auto_tool_en,
                    aa_en,
                    aa_range,
                    aa_fov,
                    aa_speed,
                    tp_aura_en,
                    tp_range,
                    ac_en,
                    ac_min_cps,
                    ac_max_cps,
                    ac_jitter,
                    scaffold_en,
                    scaffold_sprint,
                    scaffold_sched,
                    scaffold_tower,
                    block_in_en,
                    bed_aura_en,
                    bed_aura_range,
                    weather_clr,
                ) = {
                    let Ok(mut raven) = $crate::raven_bs::global_raven().lock() else {
                        return;
                    };
                    if on_ground {
                        raven.air_ticks = 0;
                    } else {
                        raven.air_ticks = raven.air_ticks.saturating_add(1);
                    }
                    let res = (
                        raven.cfg.timer_enabled,
                        raven.cfg.timer_value,
                        raven.cfg.fullbright_enabled,
                        raven.cfg.anti_debuff_enabled,
                        raven.cfg.fast_mine_enabled,
                        raven.cfg.reach_enabled,
                        raven.cfg.reach_max,
                        raven.cfg.hitbox_enabled,
                        raven.cfg.hitbox_expand,
                        raven.cfg.delay_remover_enabled,
                        raven.cfg.fast_place_enabled,
                        raven.cfg.fast_place_delay,
                        raven.cfg.sprint_enabled,
                        raven.cfg.nofall_enabled,
                        raven.cfg.noslow_enabled,
                        raven.cfg.auto_jump_enabled,
                        raven.cfg.jump_reset_enabled,
                        raven.cfg.velocity_enabled,
                        raven.cfg.velocity_horizontal,
                        raven.cfg.velocity_vertical,
                        raven.cfg.no_hurt_cam_enabled,
                        raven.cfg.vclip_enabled,
                        raven.cfg.vclip_distance,
                        raven.cfg.stop_motion_enabled,
                        raven.cfg.stasis_enabled,
                        raven.cfg.fly_enabled,
                        raven.cfg.freecam_enabled,
                        raven.cfg.fly_mode,
                        raven.cfg.fly_speed,
                        raven.cfg.freecam_speed,
                        raven.cfg.speed_enabled,
                        raven.cfg.bhop_enabled,
                        raven.cfg.boost_enabled,
                        raven.cfg.speed_mode,
                        raven.cfg.speed_multiplier,
                        raven.cfg.long_jump_enabled,
                        raven.cfg.long_jump_boost,
                        raven.cfg.instant_stop_enabled,
                        raven.cfg.step_enabled,
                        raven.cfg.step_height,
                        raven.cfg.dolphin_enabled,
                        raven.cfg.safewalk_enabled,
                        raven.cfg.bridge_assist_enabled,
                        raven.cfg.killaura_enabled,
                        raven.cfg.killaura_range,
                        raven.cfg.killaura_min_cps,
                        raven.cfg.killaura_max_cps,
                        raven.cfg.autoblock_enabled,
                        raven.cfg.auto_swap_enabled,
                        raven.cfg.auto_tool_enabled,
                        raven.cfg.aim_assist_enabled,
                        raven.cfg.aim_assist_distance,
                        raven.cfg.aim_assist_fov,
                        raven.cfg.aim_assist_speed,
                        raven.cfg.tp_aura_enabled,
                        raven.cfg.tp_aura_range,
                        raven.cfg.autoclicker_enabled,
                        raven.cfg.autoclicker_min_cps,
                        raven.cfg.autoclicker_max_cps,
                        raven.cfg.autoclicker_jitter,
                        raven.cfg.scaffold_enabled,
                        raven.cfg.scaffold_sprint,
                        raven.cfg.scaffold_schedule,
                        raven.cfg.scaffold_tower,
                        raven.cfg.block_in_enabled,
                        raven.cfg.bed_aura_enabled,
                        raven.cfg.bed_aura_range,
                        raven.cfg.weather_clear,
                    );
                    if vclip_en {
                        raven.cfg.vclip_enabled = false;
                    }
                    if stop_mot_en {
                        raven.cfg.stop_motion_enabled = false;
                    }
                    res
                };

                // 1. Timer
                if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                    if timer_en {
                        self.clock.set_rate((20.0 * timer_val).clamp(2.0, 100.0));
                        raven.timer_was_active = true;
                    } else if raven.timer_was_active {
                        self.clock.set_rate(20.0);
                        raven.timer_was_active = false;
                    }
                }

                // 2. Fullbright & AntiDebuff
                if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                    if fullbright_en {
                        raven.fullbright_was_active = true;
                        match self
                            .potions
                            .effects
                            .list
                            .iter_mut()
                            .find(|e| e.id == "night_vision")
                        {
                            Some(existing) => existing.duration = existing.duration.max(400),
                            None => {
                                let mut nv = horizon_client::effects::EffectInstance::new(
                                    "night_vision",
                                    400,
                                    0,
                                );
                                nv.ambient = true;
                                nv.visible = false;
                                nv.show_icon = false;
                                self.potions.effects.list.push(nv);
                            }
                        }
                    } else if raven.fullbright_was_active {
                        raven.fullbright_was_active = false;
                        self.potions
                            .effects
                            .list
                            .retain(|e| !(e.id == "night_vision" && e.ambient && !e.show_icon));
                    }

                    if anti_debuff_en {
                        self.potions.effects.list.retain(|e| {
                            !matches!(e.id, "blindness" | "darkness" | "nausea" | "confusion")
                        });
                        self.screen_fx.nausea = 0.0;
                        self.screen_fx.o_nausea = 0.0;
                        self.sculk.darkness = 0.0;
                    }

                    // 3. FastMine
                    if fast_mine_en {
                        raven.fastmine_was_active = true;
                        self.destroy_delay = 0;
                        match self
                            .potions
                            .effects
                            .list
                            .iter_mut()
                            .find(|e| e.id == "haste")
                        {
                            Some(existing) => {
                                existing.duration = existing.duration.max(400);
                                existing.amplifier = existing.amplifier.max(2);
                            }
                            None => {
                                let mut hs =
                                    horizon_client::effects::EffectInstance::new("haste", 400, 2);
                                hs.ambient = true;
                                hs.visible = false;
                                hs.show_icon = false;
                                self.potions.effects.list.push(hs);
                            }
                        }
                    } else if raven.fastmine_was_active {
                        raven.fastmine_was_active = false;
                        self.potions
                            .effects
                            .list
                            .retain(|e| !(e.id == "haste" && e.ambient && !e.show_icon));
                    }
                }

                // 4. Reach & HitBox & DelayRemover
                if reach_en {
                    self.entity_reach = self.entity_reach.max(reach_max);
                    self.block_reach = self.block_reach.max(reach_max + 1.0);
                }
                if hitbox_en {
                    self.entity_reach = self.entity_reach.max(3.2 + hitbox_exp * 2.5);
                }
                if delay_rem_en {
                    self.destroy_delay = 0;
                    self.clicks.miss_time = 0;
                }

                // 5. FastPlace (places on looked-at block with 0 delay when holding touch or right click)
                if fast_place_en {
                    self.use_delay = self.use_delay.min(u32::from(fast_place_del));
                    if (self.right || self.touch.holding.is_some())
                        && self.ticks % u64::from(fast_place_del).saturating_add(1) == 0
                    {
                        self.stop_remote_mining();
                        self.use_delay = 0;
                        if let Some(hit) = self.pick() {
                            let target_cell = [
                                hit.pos[0] + hit.face[0],
                                hit.pos[1] + hit.face[1],
                                hit.pos[2] + hit.face[2],
                            ];
                            let _ = self.raven_place_block_at(target_cell, hit);
                        }
                    }
                }

                // 6. Movement Physics (Fly, Speed, BHop, Boost, LongJump, Step, Dolphin, Velocity, JumpReset)
                if let Some(p) = self.player.as_mut() {
                    if sprint_en && is_moving {
                        p.sprinting = true;
                    }
                    if nofall_en || fly_en {
                        p.fall_distance = 0.0;
                        p.ground.on_ground = true;
                    }
                    if noslow_en {
                        p.item_use_speed = Some(1.0);
                        p.item_use_slows_sprint = false;
                    }
                    if auto_jump_en {
                        p.auto_jump_enabled = true;
                    }

                    if p.health.hurt.cooldown > 0 {
                        if jump_reset_en && on_ground && p.health.hurt.cooldown >= 8 {
                            p.velocity.y = 0.42;
                        }
                        if vel_en && p.health.hurt.cooldown >= 8 {
                            let h = f64::from(vel_h) / 100.0;
                            let v = f64::from(vel_v) / 100.0;
                            p.velocity.x *= h;
                            p.velocity.z *= h;
                            p.velocity.y *= v;
                        }
                        if no_hurt_en {
                            p.health.hurt.cooldown = 0;
                        }
                    }

                    if vclip_en {
                        p.position.y += vclip_dist;
                        p.velocity.y = 0.0;
                        p.teleported = true;
                    }

                    if stop_mot_en || stasis_en {
                        p.velocity = Vec3::ZERO;
                    }

                    if fly_en || freecam_en {
                        p.flying = true;
                        let spd = if freecam_en {
                            freecam_spd * 0.48
                        } else {
                            fly_spd * 0.45
                        };
                        if is_moving {
                            p.velocity.x = dir_x * spd;
                            p.velocity.z = dir_z * spd;
                        } else {
                            p.velocity.x = 0.0;
                            p.velocity.z = 0.0;
                        }
                        p.velocity.y = match fly_mode {
                            $crate::raven_bs::FlyMode::Vanilla
                            | $crate::raven_bs::FlyMode::Motion => {
                                if jump_held {
                                    spd * 0.75
                                } else if sneak_held {
                                    -spd * 0.75
                                } else {
                                    0.0
                                }
                            }
                            $crate::raven_bs::FlyMode::Glide => {
                                if jump_held {
                                    spd * 0.65
                                } else {
                                    -0.02
                                }
                            }
                            $crate::raven_bs::FlyMode::AirWalk => {
                                p.ground.on_ground = true;
                                0.0
                            }
                        };
                    } else {
                        if !self.creative {
                            p.flying = false;
                        }
                        if speed_en || bhop_en || boost_en {
                            let mult = if boost_en {
                                speed_mult.max(1.65)
                            } else {
                                speed_mult
                            };
                            if is_moving {
                                p.sprinting = true;
                                p.velocity.x = dir_x * 0.31 * mult;
                                p.velocity.z = dir_z * 0.31 * mult;
                                if on_ground
                                    && (bhop_en
                                        || matches!(
                                            speed_mode,
                                            $crate::raven_bs::SpeedMode::BHop
                                                | $crate::raven_bs::SpeedMode::HypixelLowHop
                                        ))
                                {
                                    p.velocity.y = 0.42;
                                }
                            }
                        }

                        if long_jump_en && is_moving {
                            if on_ground {
                                p.velocity.y = 0.44;
                                let lj_spd = 0.38 * long_jump_boost;
                                p.velocity.x = dir_x * lj_spd;
                                p.velocity.z = dir_z * lj_spd;
                            } else {
                                p.velocity.y += 0.024;
                            }
                        }

                        if inst_stop_en && !is_moving && on_ground {
                            p.velocity.x = 0.0;
                            p.velocity.z = 0.0;
                        }
                    }

                    if step_en && is_moving && on_ground && horiz_col {
                        p.position.y += f64::from(step_h.clamp(1.0, 2.0));
                        p.velocity.y = 0.0;
                    }

                    if dolphin_en && in_liquid {
                        p.velocity.y = 0.14;
                        if is_moving {
                            p.velocity.x = dir_x * 0.38;
                            p.velocity.z = dir_z * 0.38;
                        }
                    }
                }

                // 7. SafeWalk & BridgeAssist
                if (safewalk_en || bridge_assist_en) && on_ground {
                    if let Some(p) = self.player.as_mut() {
                        let next_x = (p.position.x + p.velocity.x * 1.6).floor() as i32;
                        let next_y = (p.position.y - 0.5).floor() as i32;
                        let next_z = (p.position.z + p.velocity.z * 1.6).floor() as i32;
                        if self
                            .res
                            .facts()
                            .is_air(self.world.block(next_x, next_y, next_z))
                        {
                            p.velocity.x = 0.0;
                            p.velocity.z = 0.0;
                        }
                    }
                }

                // 8. Combat: KillAura, AimAssist, AutoClicker, TPAura (Sends BOTH `S::Attack` AND `S::Swing`!)
                let eye = self
                    .eye_position()
                    .unwrap_or([p_pos[0], p_pos[1] + 1.62, p_pos[2]]);
                let search_range = ka_range.max(aa_range).max(tp_range);

                let mut targets: Vec<(u32, [f64; 3], f64, String, f32, f32)> = self
                    .mobs
                    .iter()
                    .filter(|m| !m.removed && m.alive() && Some(m.id) != self.riding)
                    .filter_map(|m| {
                        let tpos = [
                            m.position.x,
                            m.position.y + f64::from(m.height()) * 0.55,
                            m.position.z,
                        ];
                        let dx = tpos[0] - eye[0];
                        let dy = tpos[1] - eye[1];
                        let dz = tpos[2] - eye[2];
                        let dist = (dx * dx + dy * dy + dz * dz).sqrt();
                        if dist <= search_range {
                            let clean_name = m
                                .kind
                                .id
                                .trim_start_matches("minecraft:")
                                .replace('_', " ");
                            Some((m.id, tpos, dist, clean_name, m.health.max(1.0), 20.0))
                        } else {
                            None
                        }
                    })
                    .collect();

                targets.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap());

                if let Some(&(mob_id, tpos, dist, ref name, hp, max_hp)) = targets.first() {
                    if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                        raven.target_info = Some((name.clone(), hp, max_hp, dist));
                    }

                    if aa_en && dist <= aa_range {
                        let desired = $crate::raven_bs::calc_angles(eye, tpos);
                        if $crate::raven_bs::wrap_deg(desired.0 - self.yaw).abs() <= aa_fov {
                            let (nyaw, npitch) = $crate::raven_bs::smooth_rot(
                                (self.yaw, self.pitch),
                                desired,
                                aa_speed,
                            );
                            self.yaw = nyaw;
                            self.pitch = npitch;
                        }
                    }

                    if tp_aura_en && dist <= tp_range && self.ticks % 6 == 0 {
                        if let Some(p) = self.player.as_mut() {
                            p.position = Vec3::new(tpos[0], tpos[1] - 0.3, tpos[2]);
                            p.teleported = true;
                        }
                    }

                    // KillAura: rotate + send `S::Attack` + `S::Swing` directly to server!
                    if ka_en && dist <= ka_range {
                        let desired = $crate::raven_bs::calc_angles(eye, tpos);
                        self.yaw = desired.0;
                        self.pitch = desired.1;
                        self.entity_reach = self.entity_reach.max(ka_range + 0.5);

                        if auto_swap_en || auto_tool_en {
                            if let Some(sword_slot) =
                                self.inventory.iter().take(9).position(|s| {
                                    s.as_ref().is_some_and(|it| {
                                        it.item.contains("sword") || it.item.contains("axe")
                                    })
                                })
                            {
                                self.selected = sword_slot;
                            }
                        }

                        let should_hit = if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                            let r = raven.rand_f32();
                            let cps = ka_min_cps + r * (ka_max_cps - ka_min_cps);
                            let delay = Duration::from_secs_f32(1.0 / cps.max(1.0));
                            if raven.last_attack.elapsed() >= delay {
                                raven.last_attack = Instant::now();
                                raven.switch_idx = raven.switch_idx.wrapping_add(1);
                                raven.lmb_clicks.push_back(raven.last_attack);
                                true
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if should_hit {
                            self.swing_arm();
                            if self.remote.is_some() {
                                let sneaking = self.remote_shift_down();
                                self.send(horizon_protocol::packets::play::Serverbound::Attack(
                                    horizon_protocol::packets::play::Attack {
                                        id: mob_id as i32,
                                        secondary: sneaking,
                                    },
                                ));
                                self.send(horizon_protocol::packets::play::Serverbound::Swing(
                                    horizon_protocol::packets::play::Swing { hand: 0 },
                                ));
                                if autoblock_en {
                                    self.remote_use();
                                }
                            } else {
                                self.attack_clicks = self.attack_clicks.saturating_add(1);
                            }
                        }
                    }
                } else if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                    raven.target_info = None;
                }

                // AutoClicker (with `S::Swing` + `S::Attack`)
                if ac_en && (self.left || self.touch.holding.is_some()) {
                    let should_click = if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                        let r = raven.rand_f32();
                        let cps = ac_min_cps + r * (ac_max_cps - ac_min_cps);
                        let delay = Duration::from_secs_f32(1.0 / cps.max(1.0));
                        if raven.last_attack.elapsed() >= delay {
                            raven.last_attack = Instant::now();
                            raven.lmb_clicks.push_back(raven.last_attack);
                            if ac_jitter > 0.0 {
                                let jx = (raven.rand_f32() - 0.5) * ac_jitter;
                                let jy = (raven.rand_f32() - 0.5) * ac_jitter;
                                self.yaw = $crate::raven_bs::wrap_deg(self.yaw + jx);
                                self.pitch = (self.pitch + jy).clamp(-90.0, 90.0);
                            }
                            true
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    if should_click {
                        self.swing_arm();
                        if let Some((idx, _, _)) = self.mob_hit_within(self.entity_reach) {
                            let mob_id = self.mobs[idx].id as i32;
                            if self.remote.is_some() {
                                let sneaking = self.remote_shift_down();
                                self.send(horizon_protocol::packets::play::Serverbound::Attack(
                                    horizon_protocol::packets::play::Attack {
                                        id: mob_id,
                                        secondary: sneaking,
                                    },
                                ));
                                self.send(horizon_protocol::packets::play::Serverbound::Swing(
                                    horizon_protocol::packets::play::Swing { hand: 0 },
                                ));
                            } else {
                                self.attack_clicks = self.attack_clicks.saturating_add(1);
                            }
                        } else if self.remote.is_some() {
                            self.send(horizon_protocol::packets::play::Serverbound::Swing(
                                horizon_protocol::packets::play::Swing { hand: 0 },
                            ));
                        }
                    }
                }

                // 9. Scaffold: places blocks directly under feet & ahead with `raven_place_block_at`
                if scaffold_en {
                    if let Some(slot) = self.inventory.iter().take(9).position(|s| {
                        s.as_ref().is_some_and(|st| {
                            st.count > 0
                                && (self.pack.assets.registry.default_state(&st.item).is_some()
                                    || self
                                        .items()
                                        .get(&st.item)
                                        .is_some_and(|info| info.block.is_some()))
                        })
                    }) {
                        self.selected = slot;
                    }

                    if scaffold_tower && jump_held {
                        if let Some(p) = self.player.as_mut() {
                            p.velocity.y = 0.42;
                        }
                    }

                    if is_moving
                        && on_ground
                        && matches!(
                            scaffold_sprint,
                            $crate::raven_bs::ScaffoldSprintMode::JumpSprint
                                | $crate::raven_bs::ScaffoldSprintMode::HypixelJumpSprint
                                | $crate::raven_bs::ScaffoldSprintMode::HypixelJump2Sprint
                                | $crate::raven_bs::ScaffoldSprintMode::HypixelJump3Sprint
                        )
                    {
                        if let Some(p) = self.player.as_mut() {
                            p.velocity.y = 0.42;
                        }
                    }

                    let air_t = $crate::raven_bs::global_raven()
                        .lock()
                        .map(|r| r.air_ticks)
                        .unwrap_or(0);
                    let can_place = match scaffold_sched {
                        $crate::raven_bs::ScaffoldScheduleMode::Normal => true,
                        $crate::raven_bs::ScaffoldScheduleMode::SimpleTelly => {
                            air_t == 0 || air_t >= 2
                        }
                        $crate::raven_bs::ScaffoldScheduleMode::Telly => air_t == 0 || air_t >= 3,
                    };

                    if can_place {
                        let y_under = (p_pos[1] - 0.95).floor() as i32;
                        let candidate_cells = [
                            [p_pos[0].floor() as i32, y_under, p_pos[2].floor() as i32],
                            [
                                (p_pos[0] + dir_x * 0.45).floor() as i32,
                                y_under,
                                (p_pos[2] + dir_z * 0.45).floor() as i32,
                            ],
                            [
                                (p_pos[0] + dir_x * 0.85).floor() as i32,
                                y_under,
                                (p_pos[2] + dir_z * 0.85).floor() as i32,
                            ],
                        ];

                        for cell in candidate_cells {
                            if let Some(hit) = self.raven_find_place_hit(cell) {
                                if self.raven_place_block_at(cell, hit) {
                                    if let Ok(mut raven) = $crate::raven_bs::global_raven().lock() {
                                        raven.blocks_placed_session =
                                            raven.blocks_placed_session.saturating_add(1);
                                    }
                                    break;
                                }
                            }
                        }
                    }
                }

                // 10. BlockIn: surrounds player with blocks
                if block_in_en {
                    let bx = p_pos[0].floor() as i32;
                    let by = p_pos[1].floor() as i32;
                    let bz = p_pos[2].floor() as i32;
                    let surround = [
                        [bx + 1, by, bz],
                        [bx - 1, by, bz],
                        [bx, by, bz + 1],
                        [bx, by, bz - 1],
                        [bx + 1, by + 1, bz],
                        [bx - 1, by + 1, bz],
                        [bx, by + 1, bz + 1],
                        [bx, by + 1, bz - 1],
                        [bx, by + 2, bz],
                    ];
                    for cell in surround {
                        if let Some(hit) = self.raven_find_place_hit(cell) {
                            if self.raven_place_block_at(cell, hit) {
                                break;
                            }
                        }
                    }
                }

                // 11. BedAura: breaks beds within range both locally & via `S::PlayerAction`
                if bed_aura_en {
                    let r = bed_aura_range.ceil() as i32;
                    let px = p_pos[0].floor() as i32;
                    let py = p_pos[1].floor() as i32;
                    let pz = p_pos[2].floor() as i32;
                    'bed_loop: for dx in -r..=r {
                        for dy in -r..=r {
                            for dz in -r..=r {
                                let bp = [px + dx, py + dy, pz + dz];
                                let st = self.world.block(bp[0], bp[1], bp[2]);
                                if self.res.facts().get(st).name.ends_with("_bed") {
                                    if self.remote.is_some() {
                                        let seq = if let Some(rem) = self.remote.as_mut() {
                                            rem.sequence += 1;
                                            rem.sequence
                                        } else {
                                            1
                                        };
                                        self.send(
                                            horizon_protocol::packets::play::Serverbound::PlayerAction(
                                                horizon_protocol::packets::play::PlayerAction {
                                                    action: 0,
                                                    pos: bp,
                                                    direction: 1,
                                                    sequence: seq,
                                                },
                                            ),
                                        );
                                        self.send(
                                            horizon_protocol::packets::play::Serverbound::PlayerAction(
                                                horizon_protocol::packets::play::PlayerAction {
                                                    action: 3,
                                                    pos: bp,
                                                    direction: 1,
                                                    sequence: seq + 1,
                                                },
                                            ),
                                        );
                                        self.send(
                                            horizon_protocol::packets::play::Serverbound::Swing(
                                                horizon_protocol::packets::play::Swing { hand: 0 },
                                            ),
                                        );
                                    }
                                    let _ = self.break_block_as_player(bp);
                                    self.swing_arm();
                                    break 'bed_loop;
                                }
                            }
                        }
                    }
                }

                // 12. Weather Clear
                if weather_clr {
                    self.world.weather.raining = false;
                    self.world.weather.thundering = false;
                    self.world.weather.rain_level = (0.0, 0.0);
                    self.world.weather.thunder_level = (0.0, 0.0);
                }
            }

            /// Projects 3D world coordinates onto the 2D HUD and draws ESP, Tracers, Keystrokes & ClickGUI
            pub(crate) fn raven_hud(&mut self, batch: &mut HudBatch) {
                let (
                    esp_mobs,
                    esp_items,
                    esp_beds,
                    esp_chests,
                    tracers,
                    nametags,
                    keystrokes,
                    saturation_hud,
                    bridge_info,
                    blocks_placed,
                ) = {
                    let Ok(mut raven) = $crate::raven_bs::global_raven().lock() else {
                        return;
                    };
                    let now = Instant::now();
                    raven
                        .lmb_clicks
                        .retain(|t| now.duration_since(*t) <= Duration::from_secs(1));
                    raven
                        .rmb_clicks
                        .retain(|t| now.duration_since(*t) <= Duration::from_secs(1));
                    (
                        raven.cfg.player_esp_enabled
                            || raven.cfg.mob_esp_enabled
                            || raven.cfg.chams_enabled,
                        raven.cfg.item_esp_enabled,
                        raven.cfg.bed_esp_enabled,
                        raven.cfg.chest_esp_enabled,
                        raven.cfg.tracers_enabled,
                        raven.cfg.nametags_enabled,
                        raven.cfg.keystrokes_enabled,
                        raven.cfg.saturation_hud_enabled,
                        raven.cfg.bridge_info_enabled,
                        raven.blocks_placed_session,
                    )
                };

                let atlas = &self.gui.atlas;
                let sw = batch.width;
                let sh = batch.height;

                let eye = self.eye_position().unwrap_or([0.0, 64.0, 0.0]);
                let inv_rot = self.camera_orientation().inverse();
                let fov_rad = (self.settings.fov as f32).clamp(30.0, 120.0).to_radians();
                let focal = 1.0 / (fov_rad * 0.5).tan();
                let aspect = sw / sh.max(1.0);

                let project = |wpos: [f64; 3]| -> Option<(f32, f32, f32)> {
                    let rel = glam::Vec3::new(
                        (wpos[0] - eye[0]) as f32,
                        (wpos[1] - eye[1]) as f32,
                        (wpos[2] - eye[2]) as f32,
                    );
                    let cam = inv_rot * rel;
                    let z_front = -cam.z;
                    if z_front <= 0.15 {
                        return None;
                    }
                    let ndc_x = (cam.x / z_front) * (focal / aspect);
                    let ndc_y = (cam.y / z_front) * focal;
                    let sx = (ndc_x + 1.0) * 0.5 * sw;
                    let sy = (1.0 - ndc_y) * 0.5 * sh;
                    if sx >= -50.0 && sx <= sw + 50.0 && sy >= -50.0 && sy <= sh + 50.0 {
                        Some((sx, sy, z_front))
                    } else {
                        None
                    }
                };

                if esp_mobs || tracers || nametags {
                    for m in &self.mobs {
                        if m.removed || !m.alive() || Some(m.id) == self.riding {
                            continue;
                        }
                        let feet = [m.position.x, m.position.y, m.position.z];
                        let head = [
                            m.position.x,
                            m.position.y + f64::from(m.height()),
                            m.position.z,
                        ];
                        if let (Some((fx, fy, dist)), Some((_, hy, _))) =
                            (project(feet), project(head))
                        {
                            let box_h = (fy - hy).abs().max(6.0);
                            let box_w = (box_h * 0.55).max(4.0);
                            let bx = fx - box_w * 0.5;
                            let by = hy.min(fy);

                            if esp_mobs {
                                let col = rgba(0, 229, 255, 230);
                                batch.fill(atlas, bx, by, box_w, 1.0, col);
                                batch.fill(atlas, bx, by + box_h, box_w, 1.0, col);
                                batch.fill(atlas, bx, by, 1.0, box_h, col);
                                batch.fill(atlas, bx + box_w, by, 1.0, box_h, col);
                            }

                            if nametags {
                                let label = format!(
                                    "{} [{:.0}m]",
                                    m.kind.id.trim_start_matches("minecraft:"),
                                    dist
                                );
                                let lw = HudBatch::text_width(atlas, &label);
                                batch.fill(
                                    atlas,
                                    fx - lw * 0.5 - 2.0,
                                    by - 11.0,
                                    lw + 4.0,
                                    10.0,
                                    rgba(0, 0, 0, 160),
                                );
                                batch.text(
                                    atlas,
                                    &label,
                                    fx - lw * 0.5,
                                    by - 10.0,
                                    rgba(255, 255, 255, 255),
                                    true,
                                );
                            }

                            if tracers {
                                let cx = sw * 0.5;
                                let cy = sh * 0.5;
                                batch.fill_quad(
                                    atlas,
                                    [
                                        [cx - 0.5, cy],
                                        [cx + 0.5, cy],
                                        [fx + 0.5, fy],
                                        [fx - 0.5, fy],
                                    ],
                                    rgba(168, 85, 247, 190),
                                );
                            }
                        }
                    }
                }

                if esp_items {
                    for item in &self.dropped {
                        let ipos = [item.position.x, item.position.y + 0.25, item.position.z];
                        if let Some((ix, iy, _)) = project(ipos) {
                            batch.fill(
                                atlas,
                                ix - 3.0,
                                iy - 3.0,
                                6.0,
                                6.0,
                                rgba(241, 196, 15, 230),
                            );
                        }
                    }
                }

                if esp_chests || esp_beds {
                    let px = eye[0].floor() as i32;
                    let py = eye[1].floor() as i32;
                    let pz = eye[2].floor() as i32;
                    if esp_chests {
                        for &(cx, cy, cz) in self.world.block_entities.keys() {
                            if (cx - px).abs() <= 32 && (cz - pz).abs() <= 32 {
                                if let Some((sx, sy, _)) = project([
                                    f64::from(cx) + 0.5,
                                    f64::from(cy) + 0.5,
                                    f64::from(cz) + 0.5,
                                ]) {
                                    batch.fill(
                                        atlas,
                                        sx - 4.0,
                                        sy - 4.0,
                                        8.0,
                                        8.0,
                                        rgba(230, 126, 34, 220),
                                    );
                                }
                            }
                        }
                    }
                    if esp_beds {
                        for dx in -16..=16 {
                            for dy in -8..=8 {
                                for dz in -16..=16 {
                                    let st = self.world.block(px + dx, py + dy, pz + dz);
                                    if self.res.facts().get(st).name.ends_with("_bed") {
                                        if let Some((sx, sy, _)) = project([
                                            f64::from(px + dx) + 0.5,
                                            f64::from(py + dy) + 0.5,
                                            f64::from(pz + dz) + 0.5,
                                        ]) {
                                            batch.fill(
                                                atlas,
                                                sx - 5.0,
                                                sy - 5.0,
                                                10.0,
                                                10.0,
                                                rgba(231, 76, 60, 230),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if keystrokes {
                    let kx = 6.0;
                    let ky = 72.0;
                    let stick = self.touch_move().unwrap_or([0.0, 0.0]);
                    let w_on = stick[1] > 0.2
                        || self.keys.held(Mapping::Forward)
                        || self.physical_keys.contains(&KeyCode::KeyW);
                    let s_on = stick[1] < -0.2
                        || self.keys.held(Mapping::Back)
                        || self.physical_keys.contains(&KeyCode::KeyS);
                    let a_on = stick[0] > 0.2
                        || self.keys.held(Mapping::Left)
                        || self.physical_keys.contains(&KeyCode::KeyA);
                    let d_on = stick[0] < -0.2
                        || self.keys.held(Mapping::Right)
                        || self.physical_keys.contains(&KeyCode::KeyD);
                    let lmb_on = self.left || self.touch.holding.is_some();
                    let rmb_on = self.right;

                    let key_box = |batch: &mut HudBatch,
                                   x: f32,
                                   y: f32,
                                   w: f32,
                                   h: f32,
                                   lbl: &str,
                                   active: bool| {
                        let bg = if active {
                            rgba(168, 85, 247, 210)
                        } else {
                            rgba(20, 20, 30, 175)
                        };
                        batch.fill(atlas, x, y, w, h, bg);
                        let tw = HudBatch::text_width(atlas, lbl);
                        batch.text(
                            atlas,
                            lbl,
                            x + (w - tw) * 0.5,
                            y + (h - 8.0) * 0.5,
                            rgba(255, 255, 255, 255),
                            true,
                        );
                    };

                    key_box(batch, kx + 18.0, ky, 16.0, 16.0, "W", w_on);
                    key_box(batch, kx, ky + 18.0, 16.0, 16.0, "A", a_on);
                    key_box(batch, kx + 18.0, ky + 18.0, 16.0, 16.0, "S", s_on);
                    key_box(batch, kx + 36.0, ky + 18.0, 16.0, 16.0, "D", d_on);
                    key_box(batch, kx, ky + 36.0, 25.0, 14.0, "LMB", lmb_on);
                    key_box(batch, kx + 27.0, ky + 36.0, 25.0, 14.0, "RMB", rmb_on);
                }

                if saturation_hud {
                    if let Some(p) = self.player.as_ref() {
                        batch.text(
                            atlas,
                            &format!("Sat: {:.1}", p.health.food.saturation),
                            sw * 0.5 + 12.0,
                            sh - 49.0,
                            rgba(241, 196, 15, 255),
                            true,
                        );
                    }
                }
                if bridge_info {
                    batch.text(
                        atlas,
                        &format!("Blocks Placed: {}", blocks_placed),
                        sw * 0.5 - 38.0,
                        sh - 62.0,
                        rgba(46, 204, 113, 255),
                        true,
                    );
                }

                $crate::raven_bs::draw_hud_and_clickgui(batch, atlas);
            }
        }
    };
}