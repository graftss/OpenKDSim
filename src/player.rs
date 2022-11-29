use crate::{delegates::DelegatesRef, gamestate::GameState, mission::state::MissionState};

use self::{
    camera::Camera, constants::MAX_PLAYERS, input::Input, katamari::Katamari, prince::Prince,
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

/// Initialization
impl Player {
    pub fn init(
        &mut self,
        player: u8,
        delegates: &DelegatesRef,
        mission_state: &MissionState,
        override_init_size: f32,
    ) {
        // first initialize the katamari
        let init_pos = &mission_state.mission_config.init_kat_pos[player as usize];
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

    /// Check if the player needs to royal warp, and if so, perform the warp.
    pub fn update_royal_warp(&mut self, warp_y: f32, area: u8, mission_state: &MissionState) {
        let Player {
            katamari,
            prince,
            camera,
            ..
        } = self;

        // only run a royal warp if the katamari center is below the death plane.
        if katamari.get_center()[1] <= warp_y {
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

    pub fn update_camera_transforms(&mut self) {
        self.camera.update_transforms();
        // TODO: `camera_update_extra_matrices()` (offset 58e40)
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
                .update(&player.prince, &player.camera, mission_state);

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
