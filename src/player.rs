use crate::{
    delegates::DelegatesRef, gamestate::GameState, macros::temp_debug_log,
    mission::state::MissionState,
};

use self::{
    camera::{mode::CameraMode, Camera},
    constants::MAX_PLAYERS,
    input::Input,
    katamari::Katamari,
    prince::{Prince, PrinceViewMode},
};

pub mod camera;
pub mod constants;
pub mod input;
pub mod katamari;
pub mod prince;

#[derive(Debug, Default)]
pub struct Player {
    pub katamari: Katamari,
    pub prince: Prince,
    pub camera: Camera,
    pub input: Input,
}

impl Player {
    pub fn init(
        &mut self,
        player: u8,
        delegates: &DelegatesRef,
        mission_state: &MissionState,
        override_init_size: f32,
    ) {
        // first initialize the katamari
        let init_pos = &mut mission_state.mission_config.init_kat_pos[player as usize].clone();

        // TEMP
        init_pos[1] += 50.0;

        let init_diam = if override_init_size < 0.0 {
            mission_state.mission_config.init_diam_cm
        } else {
            override_init_size
        };

        self.katamari
            .init(player, init_diam, init_pos, delegates, mission_state);

        // then initialize the prince
        let init_angle = mission_state.mission_config.init_prince_angle[player as usize];
        self.prince.init(player, init_angle, &self.katamari);

        // then initialize the camera
        self.camera
            .init(&self.katamari, &self.prince, &mission_state.mission_config);
    }

    pub fn update_camera(&mut self, mission_state: &MissionState) {
        self.camera
            .update(&self.prince, &mut self.katamari, mission_state);
    }

    /// Check if the player needs to royal warp, and if so, perform the warp.
    pub fn update_royal_warp(&mut self, warp_y: f32, area: u8, mission_state: &MissionState) {
        let Player {
            katamari,
            prince,
            camera,
            ..
        } = self;

        // only run a royal warp if the katamari center is below the death plane.
        if katamari.get_center()[1] >= warp_y {
            return;
        }

        // only run a royal warp if the stage has royal warp destinations
        let dest = mission_state
            .stage_config
            .get_royal_warp_dest(area as usize);
        if dest.is_none() {
            return;
        }

        // update the warped player's katamari, prince, and camera.
        katamari.update_royal_warp(&dest.unwrap().kat_pos, mission_state);
        prince.update_royal_warp(katamari, dest.unwrap().prince_angle);
        camera.reset_state(katamari, prince);

        // TODO: call `vs_volume_diff_callback` delegate
    }

    /// Mimicks the `SetShootingMode` API function.
    /// While the original simulation saves the `fg` argument to the `Prince` struct, it
    /// appears to be unused.
    /// offset: 0x3d60
    pub fn set_shooting_mode(&mut self, _fg: bool, reset: bool) {
        if reset {
            self.prince.set_view_mode(PrinceViewMode::Normal);
            self.prince.set_ignore_input_timer(0);
            self.camera.set_mode(CameraMode::Normal);
        }
    }
}

pub type PlayersState = [Player; MAX_PLAYERS];

impl GameState {
    /// Update the prince and katamari controlled by the given `player`.    
    /// offset: 0x25be0
    pub fn update_prince_and_kat(&mut self, player_idx: usize) {
        let mission_state = &mut self.mission_state;
        let global = &mut self.global;
        let player = &mut self.players[player_idx];

        if global.freeze {
            player.katamari.update_collision_rays();
            // TODO: `player_update:29-31` (probably a no-op, but unclear)
        } else {
            // update the prince, then the katamari
            player.update_prince(mission_state);
            player
                .katamari
                .update(&mut player.prince, &player.camera, mission_state);

            // update the prince's transform now that the katamari is updated
            player.prince.update_transform(&player.katamari);
            // TODO: self.princes[player].update_animation(); (although animations might want to be their own struct)

            player.update_royal_warp(
                self.global.royal_warp_plane_y,
                mission_state.area,
                &mission_state,
            );
        }
    }
}
