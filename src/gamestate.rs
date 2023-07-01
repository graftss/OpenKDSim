use std::{cell::RefCell, rc::Rc};

use crate::{
    delegates::Delegates,
    global::GlobalState,
    macros::panic_log,
    mission::{config::MissionConfig, state::MissionState, vsmode::VsModeState, GameMode},
    mono_data::MonoData,
    player::{Player, PlayersState},
    props::{prop::AddPropArgs, PropsState},
};

#[derive(Debug, Default)]
pub struct GameState {
    /// State unique to a particular player.
    pub players: PlayersState,

    /// Global, mutable state (like the number of ticks that have occurred
    /// so far in the current mission).
    pub global: GlobalState,

    /// State relating to props.
    pub props: PropsState,

    /// State relating to the mission in progress.
    pub mission_state: MissionState,

    /// Delegates which call back into unity code.
    pub delegates: Rc<RefCell<Delegates>>,

    /// Constant, geometric data relating to props that's passed to the
    /// simulation from unity (e.g. prop collision meshes, prop random
    /// roam zones).
    pub mono_data: MonoData,
}

impl GameState {
    pub fn reset(&mut self) {
        self.players = PlayersState::default();
        self.global = GlobalState::default();
        self.props.reset();
        self.mission_state = MissionState::default();
        self.mono_data = MonoData::default();

        // TODO: find a better place to put this
        self.props.delegates = Some(self.delegates.clone());
    }

    pub fn get_player(&self, player_idx: usize) -> &Player {
        self.players.get(player_idx).unwrap()
    }

    pub fn get_mut_player(&mut self, player_idx: usize) -> &mut Player {
        self.players.get_mut(player_idx).unwrap()
    }

    /// The `MissionConfig` for the current mission.
    pub fn get_mission_config(&self) -> &MissionConfig {
        &self.mission_state.mission_config
    }

    /// Mimicks `SetKatamariSpeed` API function.
    /// Note that the four acceleration values are unused (as they are in the
    /// original simulation).
    pub fn set_katamari_speed(
        &mut self,
        forw_s: f32,
        side_s: f32,
        backw_s: f32,
        boost_s: f32,
        _forw_a: f32,
        _side_a: f32,
        _back_a: f32,
        _boost_a: f32,
        rot_s: f32,
        limit_y: f32,
        cam_x: f32,
        cam_y: f32,
        cam_z: f32,
    ) {
        self.global.royal_warp_plane_y = limit_y * 100.0;
        self.players[0]
            .katamari
            .set_speed(forw_s, side_s, backw_s, boost_s);
        self.players[0].prince.set_global_turn_speed(rot_s);
        self.players[0].camera.set_delay(cam_x, cam_y, cam_z);
    }

    /// Mimicks the `SetGameTime` API function.
    pub fn set_game_time(
        &mut self,
        game_time_ms: i32,
        remain_time_ticks: i32,
        freeze: i32,
        cam_eff_1P: i32,
    ) {
        self.global.game_time_ms = game_time_ms;
        self.global.remain_time_ticks = remain_time_ticks;
        self.global.freeze = freeze > 0;
        self.get_mut_player(0 as usize)
            .camera
            .set_cam_eff_1P(cam_eff_1P);
    }

    /// Mimicks the `GetPrince` API function.
    pub fn get_prince(
        &self,
        player_idx: usize,
        xx: &mut f32,
        xy: &mut f32,
        xz: &mut f32,
        yx: &mut f32,
        yy: &mut f32,
        yz: &mut f32,
        zx: &mut f32,
        zy: &mut f32,
        zz: &mut f32,
        tx: &mut f32,
        ty: &mut f32,
        tz: &mut f32,
        view_mode: &mut i32,
        _face_mode: &mut i32,
        alarm_mode: &mut i32,
        alarm_type: &mut i32,
        hit_water: &mut i32,
        map_loop_rate: &mut f32,
    ) {
        let player = self.get_player(player_idx);
        let prince = &player.prince;
        prince.get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz);
        *view_mode = prince.get_view_mode() as i32;

        // TODO_LOW: update `face_mode`

        let katamari = &player.katamari;
        katamari.get_alarm(alarm_mode, alarm_type);
        *hit_water = katamari.is_in_water() as i32;

        *map_loop_rate = self.global.map_loop_rate;
    }

    /// Mimicks the `SetGameStart` API function.
    /// Note that in the actual simulation, the "area" argument is unused.
    pub fn set_game_start(&mut self, player_idx: usize, _area: u8) {
        self.global.freeze = false;
        self.global.map_change_mode = false;
        self.get_mut_player(player_idx)
            .prince
            .set_ignore_input_timer(0);
    }

    /// Mimicks the `SetAreaChange` API function.
    pub fn set_area_change(&mut self, player_idx: usize) {
        self.global.freeze = true;
        self.global.map_change_mode = true;

        let player = &mut self.players[player_idx];
        player.prince.set_ignore_input_timer(-1);
        player.katamari.set_immobile(&self.mission_state);
    }

    /// Mimicks the `SetMapChangeMode` API function.
    pub fn set_map_change_mode(&mut self, map_change_mode: i32) {
        self.global.map_change_mode = map_change_mode != 0;
    }

    /// Mimicks the `GetRadiusTargetPercent` API function.
    pub fn get_radius_target_percent(&self, player_idx: usize) -> f32 {
        let player = self.get_player(player_idx);
        let kat = &player.katamari;
        let mission = &self.mission_state;
        let mission_config = &mission.mission_config;

        let init_rad = kat.get_init_radius();
        let curr_rad = kat.get_radius();

        let goal_rad = mission_config.goal_diam_cm / 2.0;

        (curr_rad - init_rad) / (goal_rad - init_rad)
    }

    /// Mimicks the `GetPropAttached` API function.
    /// Returns the number of 3-byte prop statuses written to `out`.
    pub unsafe fn get_props_attach_status(&self, out: *mut u8) -> i32 {
        let kat_diam_int = self.get_player(0).katamari.get_diam_int();
        self.props.get_attach_statuses(out, kat_diam_int)
    }

    // Mimicks the `MonoInitStart` API function.
    pub unsafe fn mono_init_start(
        &mut self,
        mono_data: *const u8,
        mission: u8,
        area: u8,
        stage: u8,
        _kadai_flag: bool,
        _clear_flag: bool,
        _end_flag: bool,
    ) {
        self.global.mono_init_start();
        self.mission_state.mono_init_start(mission, area, stage);

        // read the mission's `MonoData` data from the `mono_data` raw pointer.
        self.mono_data.init(mono_data);

        // TODO_PROPS: init subobjects
        // TODO_PROPS: init comment prop groups
        // TODO_PROPS: init random prop groups
        // TODO_PROPS: init generated props
    }

    /// Mimicks the `MonoInitAddProp` API function.
    /// Creates a new prop using the provided arguments.
    /// Returns the control index of the created prop.
    pub fn add_prop(&mut self, args: &AddPropArgs) -> i32 {
        let ctrl_idx = self.global.get_next_ctrl_idx();
        let area = self.mission_state.area;
        let mono_data = self.mono_data.props.get(args.name_idx as usize);

        self.props.add_prop(ctrl_idx, args, area, mono_data);

        ctrl_idx as i32
    }

    /// Mimicks the `MonoInitAddPropSetParent` API function.
    pub fn add_prop_set_parent(&mut self, child_ctrl_idx: i32, parent_ctrl_idx: i32) {
        // the child prop must exist
        let child_rc = self
            .props
            .get_prop(child_ctrl_idx as usize)
            .unwrap_or_else(|| {
                panic_log!("called `add_prop_set_parent` on a nonexistent prop: {child_ctrl_idx}");
            });

        if let Some(parent_rc) = self.props.get_prop(parent_ctrl_idx as usize) {
            // adding a parent prop to the child
            let area = self.mission_state.area as u32;
            let tree_id: u32 = 1000 * area + (self.global.num_root_props as u32);

            // declare that the child has a parent
            child_rc.clone().borrow_mut().set_parent(
                &self.props,
                parent_ctrl_idx as u16,
                tree_id as u16,
            );
            parent_rc
                .clone()
                .borrow_mut()
                .add_child(&self.props, child_ctrl_idx as u16);
        } else {
            // declaring that the child prop has no parent
            child_rc.clone().borrow_mut().set_no_parent();
        }
    }

    /// Mimicks the `MonoInitEnd` API function.
    pub fn mono_init_end(&mut self) {
        // TODO_PROPS: init_cache_gemini_twins();
        self.props.global_paths.init();
        self.global.props_initialized = true;
    }

    /// Mimicks the `SetStoreFlag` API function.
    pub fn set_store_flag(&mut self, store_flag: bool) {
        self.global.store_flag = store_flag;
        self.global.kat_diam_int_on_store_flag = self.get_player(0).katamari.get_diam_int();
    }

    /// Mimicks the `ChangeNextArea` API function.
    pub fn change_next_area(&mut self) {
        let old_updating_player = self.global.updating_player;

        self.mission_state.area += 1;
        self.mission_state.stage_area += 1;

        if self.mission_state.is_vs_mode {
            // TODO_VS: vs mode crap
        } else {
            self.props.change_next_area(self.mission_state.area)
        }

        self.global.updating_player = old_updating_player;
    }

    /// Mimicks the `Init` API function.
    pub fn init(&mut self, player_idx: usize, override_init_size: f32, mission_idx: u8) {
        let mission_state = &mut self.mission_state;
        let mission_config = &mission_state.mission_config;

        mission_state.mission = mission_idx.into();
        mission_state.vs_mission_idx = mission_state.mission.vs_mission_idx();
        mission_state.is_vs_mode = mission_state.vs_mission_idx.is_some();

        // TODO: `init_simulation`:31-97, not sure what this is for
        self.global.freeze = false;

        // TODO: `init_simulation_subroutine_1`: 0x263c0

        self.global.detaching_props_from_kat = false;
        self.global.store_flag = false;

        // TODO: `init_simulation_subroutine_2`: 0x6740

        // compute how small props need to be relative to the katamari
        // before they're destroyed as they become invisible.
        self.global.invis_prop_diam_ratio_to_destroy = self
            .props
            .params
            .compute_destroy_invis_diam_ratio(&mission_config);

        // initialize the player (katamari, prince, camera)
        self.players[player_idx].init(
            player_idx as u8,
            &self.delegates,
            mission_state,
            override_init_size,
        );

        self.global.map_loop_rate = 0.0;

        // TODO: `set_global_angle??(is_vs_mode ? 70.0 : 48.0): 0x59270 (this is probably a no-op)
        // TODO: `init_simulation`:127-277, this may be a no-op

        self.global.game_time_ms = 0;
        self.global.map_change_mode = false;

        // TODO: `init_simulation`:282-284, 290-291

        let gamemode = mission_state.gamemode;
        if gamemode == GameMode::Tutorial {
            // TODO: `init_simulation:293-325` (tutorial crap)
        }

        if mission_state.is_vs_mode {
            // initialize the vs mode state, presumably
            self.mission_state.vsmode = Some(VsModeState { timer_0x10bf10: 0 });
        } else {
            // TODO: `init_simulation`:333-366, initialize somethings coming callbacks?
        }

        // TODO: `prince_init_animation()`
        // TODO: `Init`: 21-51, initialize ending stuff
    }

    /// Mimicks the `Tick` API function.
    pub fn tick(&mut self, _delta: f32) {
        // temp_debug_log!("tick");
        let is_vs_mode = self.mission_state.is_vs_mode;

        self.global.ticks += 1;

        // TODO_STOREFLAG: `update_game:23-89` (if store flag is on)
        self.global.updating_player = 0;
        self.props.update(&self.players[0], &self.mission_state);
        // TODO: `update_game:93-101` (but put this in `props_update`)
        // TODO_TUTORIAL: `tutorial_update_flags`

        // update the first player
        self.update_prince_and_kat(0);

        if is_vs_mode {
            // if vs mode, update the second player, then update vsmode-specific stuff
            self.global.updating_player = 1;
            self.update_prince_and_kat(1);
            // TODO: `vsmode_update()`
        }

        // TODO: `camera_update_transforms()`
        // TODO: `camera_update_extra_matrices()`

        self.global.updating_player = 0;
        // TODO: `self.cameras[0].update()`

        if !is_vs_mode {
            let diam_m = self.players[0].katamari.get_diam_m();
            let _radius_ish = if diam_m >= 1.0 {
                ((diam_m - 1.0) - 0.5) + 1.0
            } else {
                diam_m
            };
            // TODO: update prop fadeout alpha crap
        } else {
            self.global.updating_player = 1;
            self.players[1].update_camera(&self.mission_state);
        }

        self.global.updating_player = 0;
        if self.get_player(0).camera.preclear.get_enabled() {
            // TODO_PRECLEAR: `update_game:142-173` (update preclear mode camera)
        }

        self.players[0].update_camera(&self.mission_state);

        if !is_vs_mode {
            // TODO: `update_game:176-255` (selectively update props based on their alpha)
        } else {
            // TODO: `update_game:258-267` (idk vs mode crap)
        }

        // TODO: `update_game:269` (keep separate running global count of # of attached props, because reasons)
        //                         (don't do this)
    }
}
