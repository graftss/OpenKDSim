use std::{cell::RefCell, rc::Rc};

use crate::{
    camera::Camera,
    constants::MAX_PLAYERS,
    delegates::Delegates,
    ending::EndingState,
    global::GlobalState,
    input::Input,
    katamari::Katamari,
    macros::panic_log,
    mission::{GameMode, GameType, Mission, MissionConfig},
    mono_data::MonoData,
    name_prop_config::NamePropConfig,
    preclear::PreclearState,
    prince::Prince,
    prop::{AddPropArgs, Prop, PropRef},
    prop_motion::GlobalPathState,
    simulation_params::SimulationParams,
    stage::StageConfig,
    tutorial::TutorialState,
    vsmode::VsModeState,
};

#[derive(Debug, Default)]
pub struct GameState {
    pub global: GlobalState,
    pub katamaris: [Katamari; MAX_PLAYERS],
    pub princes: [Prince; MAX_PLAYERS],
    pub cameras: [Camera; MAX_PLAYERS],
    pub inputs: [Input; MAX_PLAYERS],
    pub props: Vec<PropRef>,
    pub global_paths: Vec<GlobalPathState>,
    pub preclear: PreclearState,
    pub tutorial: TutorialState,
    pub vsmode: VsModeState,
    pub ending: EndingState,
    pub delegates: Delegates,
    pub mono_data: MonoData,
    pub sim_params: SimulationParams,
}

impl GameState {
    pub fn borrow_katamari(&self, player: i32) -> &Katamari {
        &self.katamaris[player as usize]
    }

    pub fn borrow_mut_katamari(&mut self, player: i32) -> &mut Katamari {
        &mut self.katamaris[player as usize]
    }

    pub fn borrow_prince(&self, player: i32) -> &Prince {
        &self.princes[player as usize]
    }

    pub fn borrow_mut_prince(&mut self, player: i32) -> &mut Prince {
        &mut self.princes[player as usize]
    }

    pub fn borrow_camera(&self, player: i32) -> &Camera {
        &self.cameras[player as usize]
    }

    pub fn borrow_mut_camera(&mut self, player: i32) -> &mut Camera {
        &mut self.cameras[player as usize]
    }

    pub fn borrow_input(&self, player: i32) -> &Input {
        &self.inputs[player as usize]
    }

    pub fn borrow_mut_input(&mut self, player: i32) -> &mut Input {
        &mut self.inputs[player as usize]
    }

    pub fn read_prop_ref(&self, ctrl_idx: i32) -> &PropRef {
        &self.props[ctrl_idx as usize]
    }

    pub fn write_prop_ref(&mut self, ctrl_idx: i32) -> &mut PropRef {
        &mut self.props[ctrl_idx as usize]
    }

    /// The `MissionConfig` for the current mission.
    pub fn current_mission_config(&self) -> Option<&MissionConfig> {
        self.global.mission.map(MissionConfig::get)
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
        self.cameras[0 as usize].set_cam_eff_1P(cam_eff_1P);
    }

    /// Mimicks the `GetPrice` API function.
    pub fn get_prince(
        &self,
        player: i32,
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
        let prince = self.borrow_prince(player);
        prince.get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz);
        *view_mode = prince.get_view_mode() as i32;

        // TODO: update `face_mode`

        let katamari = self.borrow_katamari(player);
        katamari.get_alarm(alarm_mode, alarm_type);
        *hit_water = katamari.is_in_water() as i32;

        *map_loop_rate = self.global.map_loop_rate;
    }

    /// Mimicks the `SetGameStart` API function.
    /// Note that in the actual simulation, the "area" argument is unused.
    pub fn set_game_start(&mut self, player: i32, _area: i32) {
        self.global.freeze = false;
        self.global.map_change_mode = false;
        self.borrow_mut_prince(player).set_ignore_input_timer(0);
    }

    /// Mimicks the `SetAreaChange` API function.
    pub fn set_area_change(&mut self, player: i32) {
        self.global.freeze = true;
        self.global.map_change_mode = true;
        self.borrow_mut_prince(player).set_ignore_input_timer(-1);
        self.borrow_mut_katamari(player).set_immobile();
    }

    /// Mimicks the `SetMapChangeMode` API function.
    pub fn set_map_change_mode(&mut self, map_change_mode: i32) {
        self.global.map_change_mode = map_change_mode != 0;
    }

    /// Mimicks the `GetRadiusTargetPercent` API function.
    pub fn get_radius_target_percent(&self, player: i32) -> f32 {
        let kat = self.borrow_katamari(player);
        let init_rad = kat.get_init_radius();
        let curr_rad = kat.get_radius();

        let mission_conf = MissionConfig::get(self.global.mission.unwrap());
        let goal_rad = mission_conf.goal_diam_cm / 2.0;

        (curr_rad - init_rad) / (goal_rad - init_rad)
    }

    /// Mimicks the `GetPropAttached` API function.
    /// Returns the number of 3-byte prop statuses written to `out`.
    pub unsafe fn get_props_attach_status(&self, out: *mut u8) -> i32 {
        let kat_diam_int = self.borrow_katamari(0).get_diam_int();
        let mut num_props = 0;

        for (ctrl_idx, prop_ref) in self.props.iter().enumerate() {
            let prop = prop_ref.borrow();
            if !prop.is_initialized() {
                break;
            }

            let status = out.offset((ctrl_idx * 3).try_into().unwrap());
            prop.get_attach_status(status, kat_diam_int);
            num_props += 1;
        }

        num_props
    }

    // Mimicks the `MonoInitStart` API function.
    pub unsafe fn mono_init_start(
        &mut self,
        mono_data: *const u8,
        mission: i32,
        area: i32,
        stage: i32,
        _kadai_flag: i32,
        _clear_flag: i32,
        _end_flag: i32,
    ) {
        self.global.mono_init_start(
            mission.try_into().unwrap(),
            area.try_into().unwrap(),
            stage.try_into().unwrap(),
        );

        // read the mission's `MonoData` data from the `mono_data` raw pointer.
        self.mono_data.init(mono_data);

        // TODO: init subobjects
        // TODO: init comment prop groups
        // TODO: init random prop groups
        // TODO: init generated props
    }

    // Mimicks the `MonoInitAddProp` API function.
    pub fn add_prop(&mut self, args: AddPropArgs) -> i32 {
        let prop = Prop::new(self, &args);
        let result = prop.borrow().get_ctrl_idx().into();
        self.props.push(prop);
        result
    }

    // Mimicks the `MonoInitAddPropSetParent` API function.
    pub fn add_prop_set_parent(&mut self, ctrl_idx: i32, parent_ctrl_idx: i32) {
        let child_rc = self.props.get(ctrl_idx as usize).unwrap_or_else(|| {
            panic_log!("called `add_prop_set_parent` on a nonexistent prop: {ctrl_idx}");
        });
        if let Some(parent_rc) = self.props.get(parent_ctrl_idx as usize) {
            // adding a parent prop to the child
            let weak_parent_ref = Rc::<RefCell<Prop>>::downgrade(&parent_rc);

            let area: u32 = self.global.area.unwrap().into();
            let tree_id: u32 = 1000 * area + (self.global.num_root_props as u32);

            // declare that the child has a parent
            child_rc
                .clone()
                .borrow_mut()
                .set_parent(weak_parent_ref, tree_id.try_into().unwrap());
            parent_rc.clone().borrow_mut().add_child(child_rc.clone());
        } else {
            // declaring that the child prop has no parent
            child_rc.clone().borrow_mut().set_no_parent();
        }
    }

    /// Mimicks the `MonoInitEnd` API function.
    pub fn mono_init_end(&mut self) {
        // TODO: init_cache_gemini_twins();
        GlobalPathState::init(&mut self.global_paths);
        self.global.props_initialized = true;
    }

    /// Mimicks the `MonoGetPlacementMonoDataName` API function.
    pub unsafe fn get_internal_prop_name(&self, ctrl_idx: i32) -> *const u8 {
        let name_idx = self
            .props
            .get(ctrl_idx as usize)
            .map_or(0, |prop| prop.clone().borrow().get_name_idx());

        NamePropConfig::get(name_idx.into()).internal_name.as_ptr()
    }

    /// Mimicks the `SetCameraMode` API function.
    pub fn set_camera_mode(&mut self, player: i32, mode: i32) {
        if let Some(camera) = self.cameras.get_mut(player as usize) {
            camera.set_mode(mode.into());
        }
    }

    /// Mimicks the `SetCameraCheckScaleUp` API function.
    pub fn set_camera_check_scale_up(&mut self, player: i32, flag: bool) {
        if let Some(camera) = self.cameras.get_mut(player as usize) {
            camera.check_scale_up(flag);
        }
    }

    /// Mimicks the `SetStoreFlag` API function.
    pub fn set_store_flag(&mut self, store_flag: bool) {
        self.global.store_flag = store_flag;
        self.global.kat_diam_int_on_store_flag = self.borrow_katamari(0).get_diam_int();
    }

    /// Mimicks the `ChangeNextArea` API function.
    pub fn change_next_area(&mut self) {
        let old_updating_player = self.global.updating_player;

        self.global.area.map(|v| v + 1);
        self.global.stage_area += 1;
        let new_area = self.global.area.unwrap();

        if self.global.is_vs_mode {
            // TODO: vs mode crap
        } else {
            // destroy props which have the new area as their "display off" area.
            self.props.retain(|prop_ref| {
                let prop_cell = prop_ref.clone();
                let prop = prop_cell.borrow_mut();
                prop.check_destroy_on_area_load(new_area)
            });
        }

        self.global.updating_player = old_updating_player;
    }

    /// Mimicks the `Init` API function.
    pub fn init(&mut self, player_i32: i32, override_init_size: f32, mission: u32) {
        let player: u8 = player_i32.try_into().unwrap();

        self.global.is_vs_mode = Mission::is_vs_mode(mission);
        if self.global.is_vs_mode {
            self.global.vs_mission_idx = mission - Mission::MIN_VS_MODE;
        }

        // TODO: `init_simulation`:31-97, not sure what this is for
        self.global.freeze = false;
        self.global.mission = Some(mission.into());

        // TODO: `init_simulation_subroutine_1`: 0x263c0

        self.global.detaching_props_from_kat = false;
        self.global.store_flag = false;

        // TODO: `init_simulation_subroutine_2`: 0x6740

        let mission_config = MissionConfig::get(self.global.mission.unwrap());

        // compute how small props need to be before they're destroyed at alpha 0
        self.global.prop_diam_ratio_destroy_when_invis =
            if mission_config.game_type == GameType::ClearProps {
                self.sim_params.destroy_prop_diam_ratio_clearprops
            } else if mission_config.keep_smaller_props_alive {
                self.sim_params.destroy_prop_diam_ratio_reduced
            } else {
                self.sim_params.destroy_prop_diam_ratio_normal
            };

        // initialize the katamari, the prince, and the camera (in that order)
        self.init_katamari(player, mission_config, override_init_size);
        self.init_prince(player, mission_config);
        self.cameras[player as usize].init(
            &self.katamaris[player as usize],
            &self.princes[player as usize],
        );

        self.global.map_loop_rate = 0.0;

        // TODO: `set_global_angle??(is_vs_mode ? 70.0 : 48.0): 0x59270 (this is probably a no-op)
        // TODO: `init_simulation`:127-277, this may be a no-op

        self.global.game_time_ms = 0;
        self.global.map_change_mode = false;

        // TODO: `init_simulation`:282-284, 290-291

        let gamemode = self.global.gamemode.unwrap();
        if gamemode == GameMode::Tutorial {
            // TODO: `init_simulation:293-325` (tutorial crap)
        }

        if self.global.is_vs_mode {
            self.vsmode.timer_0x10bf10 = 0;
        } else {
            // TODO: `init_simulation`:333-366, initialize somethings coming callbacks?
        }

        // TODO: `prince_init_animation()`
        // TODO: `Init`: 21-51, initialize ending stuff
    }

    fn init_katamari(
        &mut self,
        player: u8,
        mission_config: &MissionConfig,
        override_init_size: f32,
    ) {
        let init_pos = &mission_config.init_kat_pos[player as usize];
        let init_diam = if override_init_size < 0.0 {
            mission_config.init_diam_cm
        } else {
            override_init_size
        };

        let kat = &mut self.katamaris[player as usize];
        kat.init(player, init_diam, init_pos, &self.sim_params);
    }

    fn init_prince(&mut self, player: u8, mission_config: &MissionConfig) {
        let prince = &mut self.princes[player as usize];
        let init_angle = mission_config.init_prince_angle[player as usize];

        prince.init(player, init_angle, &self.katamaris[player as usize]);
    }

    /// Mimicks the `Tick` API function.
    pub fn tick(&mut self, _delta: f32) {
        let is_vs_mode = self.global.is_vs_mode;

        self.global.ticks += 1;

        // `update_game()`
        // TODO: `update_game:23-89` (if store flag is on, which seems to be irrelevant)
        self.global.updating_player = 0;
        // TODO: `props_update()`
        // TODO: `update_game:93-101` (but put this in `props_update`)
        // TODO: `tutorial_update_flags`

        // update the first player
        self.update_player(0);

        if is_vs_mode {
            // if vs mode, update the second player, then update vsmode-specific stuff
            self.global.updating_player = 1;
            self.update_player(1);
            // TODO: `vsmode_update()`
        }

        // TODO: `camera_update_transforms()`
        // TODO: `camera_update_extra_matrices()`

        self.global.updating_player = 0;
        // TODO: `self.cameras[0].update()`

        if !is_vs_mode {
            let diam_m = self.katamaris[0].get_diam_m();
            let _radius_ish = if diam_m >= 1.0 {
                ((diam_m - 1.0) - 0.5) + 1.0
            } else {
                diam_m
            };
            // TODO: update prop fadeout alpha crap
        } else {
            self.global.updating_player = 1;
            // TODO: self.cameras[1].update()`
        }

        self.global.updating_player = 0;
        if self.preclear.get_enabled() {
            // TODO: `update_game:142-173` (update preclear mode camera)
        }

        // TODO: `camera_update_transforms()`
        // TODO: `camera_update_extra_matrices()`

        if !is_vs_mode {
            // TODO: `update_game:176-255` (selectively update props based on their alpha)
        } else {
            // TODO: `update_game:258-267` (idk vs mode crap)
        }

        // TODO: `update_game:269` (keep separate running global count of # of attached props, because reasons)
        //                         (don't do this)
    }

    /// Update the prince and katamari controlled by the given `player`.    
    /// offset: 0x25be0
    fn update_player(&mut self, player: usize) {
        if self.global.freeze {
            self.katamaris[player].update_collision_rays();
            // TODO: `player_update:29-31` (probably a no-op, but unclear)
        } else {
            // update the prince, then the katamari
            self.update_prince(player);
            self.katamaris[player].update();

            // update the prince's transform now that the katamari is updated
            self.princes[player].update_transform(&self.katamaris[player]);
            // TODO: self.princes[player].update_animation(); (although animations might want to be their own struct)
            self.update_royal_warp(player);
        }
    }

    fn update_royal_warp(&mut self, player: usize) {
        let katamari = &mut self.katamaris[player];

        // only run a royal warp if the katamari center is below the death plane.
        if katamari.get_center()[2] <= self.global.royal_warp_plane_y {
            return;
        }

        // only run a royal warp if the stage has royal warp destinations
        let stage_config = StageConfig::get(self.global.stage.unwrap());
        let dest = stage_config.get_royal_warp_dest(self.global.area.unwrap() as usize);
        if dest.is_none() {
            return;
        }

        let prince = &mut self.princes[player];

        // update the warped player's katamari, prince, and camera.
        katamari.update_royal_warp(&dest.unwrap().kat_pos);
        prince.update_royal_warp(katamari, dest.unwrap().prince_angle);
        self.cameras[player].reset_state(katamari, prince);

        // TODO: call `vs_volume_diff_callback` delegate
    }
}
