use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy)]
pub struct MissionMoveType {
    pub default_action: u16,
    pub path_idx: u16,
    pub behavior: i16,
}

lazy_static! {
    pub static ref MISSION_MOVE_TYPES: Vec<Vec<MissionMoveType>> = vec![
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 2
            },
        ],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 2
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 14
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 29
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: 20
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: 13
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 14
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 15
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 19
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 2
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: 29
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 16
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
        ],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 28
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 20
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 39
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 97,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 98,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 99,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 100,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 101,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 102,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 103,
                behavior: 28
            },
        ],
        vec![
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 2
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 7
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 28
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: 8
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: 19
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: 39
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 97,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 98,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 99,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 100,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 101,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 102,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 103,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 104,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 105,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 106,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 107,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 108,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 14
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 109,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 110,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 111,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 112,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 113,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: 29
            },
        ],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 1,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: 20
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 14
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: -1
            },
        ],
        vec![],
        vec![
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 1,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 28
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 1,
                behavior: 34
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 28
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 1,
                behavior: 34
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 7
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: 28
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 19
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 97,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 98,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 99,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 100,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 101,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 102,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 103,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 104,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 105,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 106,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 107,
                behavior: 2
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 97,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 98,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 99,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 100,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 101,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 102,
                behavior: 19
            },
        ],
        vec![],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 97,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 98,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 99,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 100,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 101,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 102,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 103,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 104,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 105,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 106,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 107,
                behavior: 2
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: 13
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: 13
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 14
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 15
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 19
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: -1
            },
        ],
        vec![
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 2
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 7
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 28
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: 8
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: 39
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: 19
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: 39
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 2
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 5
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 7,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 8,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 9,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 10,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 11,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 12,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 13,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 14,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 15,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 17,
                behavior: 39
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 18,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 19,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 20,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 21,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 22,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 23,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 24,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 25,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 26,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 27,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 28,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 29,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 30,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 31,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 32,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 33,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 34,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 35,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 36,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 37,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 38,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 39,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 40,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 41,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 42,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 43,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 44,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 45,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 46,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 47,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 48,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 49,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 50,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 51,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 52,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 53,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 54,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 55,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 56,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 57,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 58,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 59,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 60,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 61,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 62,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 63,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 64,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 65,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 66,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 67,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 68,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 37
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 69,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 70,
                behavior: 29
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 16,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 71,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 72,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 73,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 74,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 75,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 76,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 77,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 78,
                behavior: 39
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 79,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 80,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 81,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 82,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 83,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 84,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 85,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 9
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 86,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 87,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 88,
                behavior: 28
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 89,
                behavior: -1
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 90,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 91,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 92,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 93,
                behavior: 36
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 94,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 95,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 96,
                behavior: -1
            },
        ],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 20
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 31
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 22
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 25
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 0
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 3
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 4
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 6
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 35
            },
            MissionMoveType {
                default_action: 0,
                path_idx: 0,
                behavior: 30
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 20
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 20
            },
        ],
        vec![
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 21
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 34
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 0,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 1,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 2,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 3,
                behavior: -1
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 32
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: -1
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 5,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 6,
                behavior: 19
            },
            MissionMoveType {
                default_action: 1,
                path_idx: 4,
                behavior: 20
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 11
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 12
            },
            MissionMoveType {
                default_action: 2,
                path_idx: 0,
                behavior: 24
            },
        ],
    ];
}
