use crate::{
    camera::Camera,
    constants::MAX_PLAYERS,
    delegates::Delegates,
    ending::EndingState,
    global::GlobalState,
    input::Input,
    katamari::Katamari,
    mission::MissionConfig,
    mono_data::MonoData,
    preclear::PreclearState,
    prince::Prince,
    prop::{AddPropArgs, Prop},
};

#[derive(Debug, Default)]
pub struct GameState {
    pub global: GlobalState,
    pub katamaris: [Katamari; MAX_PLAYERS],
    pub princes: [Prince; MAX_PLAYERS],
    pub cameras: [Camera; MAX_PLAYERS],
    pub inputs: [Input; MAX_PLAYERS],
    pub props: Vec<Prop>,
    pub preclear: PreclearState,
    pub ending: EndingState,
    pub delegates: Delegates,
    pub mono_data: MonoData,
}

impl GameState {
    pub fn read_katamari(&self, player: i32) -> &Katamari {
        &self.katamaris[player as usize]
    }

    pub fn borrow_mut_katamari(&mut self, player: i32) -> &mut Katamari {
        &mut self.katamaris[player as usize]
    }

    pub fn read_prince(&self, player: i32) -> &Prince {
        &self.princes[player as usize]
    }

    pub fn borrow_mut_prince(&mut self, player: i32) -> &mut Prince {
        &mut self.princes[player as usize]
    }

    pub fn borrow_camera(&self, player: i32) -> &Camera {
        &self.cameras[player as usize]
    }

    pub fn write_camera(&mut self, player: i32) -> &mut Camera {
        &mut self.cameras[player as usize]
    }

    pub fn read_input(&self, player: i32) -> &Input {
        &self.inputs[player as usize]
    }

    pub fn borrow_mut_input(&mut self, player: i32) -> &mut Input {
        &mut self.inputs[player as usize]
    }

    pub fn read_prop(&self, ctrl_idx: i32) -> &Prop {
        &self.props[ctrl_idx as usize]
    }

    pub fn write_prop(&mut self, ctrl_idx: i32) -> &mut Prop {
        &mut self.props[ctrl_idx as usize]
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
        let prince = self.read_prince(player);
        prince.get_matrix(xx, xy, xz, yx, yy, yz, zx, zy, zz, tx, ty, tz);
        *view_mode = prince.get_view_mode() as i32;

        // TODO: update `face_mode`

        let katamari = self.read_katamari(player);
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
        let kat = self.read_katamari(player);
        let init_rad = kat.get_init_radius();
        let curr_rad = kat.get_radius();

        let mission_conf = MissionConfig::get(self.global.mission.unwrap());
        let goal_rad = mission_conf.goal_diam_cm / 2.0;

        (curr_rad - init_rad) / (goal_rad - init_rad)
    }

    /// Mimicks the `GetPropAttached` API function.
    /// Returns the number of 3-byte prop statuses written to `out`.
    pub unsafe fn get_props_attach_status(&self, out: *mut u8) -> i32 {
        let kat_diam_int = self.read_katamari(0).get_diam_int();
        let mut num_props = 0;

        for (ctrl_idx, prop) in self.props.iter().enumerate() {
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
        _kadaiFlag: i32,
        _clearFlag: i32,
        _endFlag: i32,
    ) {
        self.global.mono_init_start(
            mission.try_into().unwrap(),
            area.try_into().unwrap(),
            stage.try_into().unwrap(),
        );

        self.mono_data.init(mono_data);

        // TODO: init subobjects
        // TODO: init comment prop groups
        // TODO: init random prop groups
        // TODO: init generated props
    }

    // Mimicks the `MonoInitAddProp` API function.
    pub fn add_prop(&mut self, args: AddPropArgs) -> i32 {
        let prop = Prop::new(self, &args);
        let result = prop.get_ctrl_idx().into();
        self.props.push(prop);
        result
    }

    // Mimicks the `MonoInitAddPropSetParent` API function.
    pub fn add_prop_set_parent(&mut self, ctrl_idx: i32, parent_ctrl_idx: i32) {
        let child = self.props.get(ctrl_idx as usize).unwrap();

        if (parent_ctrl_idx < 0) {
            self.global.num_root_props += 1;
        }
    }
}
