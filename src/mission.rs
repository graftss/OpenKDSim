#[allow(dead_code)]

/// Game missions.
#[derive(Debug)]
pub enum Mission {
  None = 0, // king talking???
  MAS1 = 1, // MAS1
  MAS2 = 2, // MAS2
  MAS4 = 3, // MAS4
  MAS3 = 4, // MAS3
  MAS5 = 5, // MAS5
  MAS6 = 6, // MAS6
  MAS7 = 7, // MAS7
  MAS8 = 8, // MAS8
  MAS9 = 9, // MAS9
  MTM = 10, // MTM
  Cancer = 11, // cancer
  Cygnus = 12, // cygnus
  Mission13 = 13, // (unused) "50 object" debug level, broken
  Corona = 14, // corona
  Pisces = 0xF, // pisces
  Virgo = 0x10, // virgo
  Ursa = 17, // ursa major
  Gemini = 18, // gemini
  Taurus = 19, // taurus
  Mission20 = 20, // (unused) mas7 area with no objects
  NorthStar = 21, // north star
  Eternal1 = 22, // eternal 1
  Eternal2 = 23, // eternal 2
  Eternal3 = 24, // eternal 3
  Mission25ShopDemo = 25, // (unused) debug l evel with starting size 0
  Mission26 = 26, // (unused) debug level with no collision, spawn above pond in mas8
  Mission27 = 27, // (unused) mas7 area with no objects
  Tutorial = 28, // tutorial (opens with PRESS START)
  Ending = 29, // countries level, gametype N
  Mission30Load = 30, // nothing loads
  Vs0 = 0x1F,
  Vs1 = 0x20,
  Vs2 = 33,
  Vs3 = 34, // vs level with magazine bridge
  Vs4 = 35,
  Vs5 = 36,
  Vs6 = 37,
  Vs7 = 38,
  GameShow = 39, // nothing loads
  Test0 = 40, // nothing loads
  Test1 = 41, // nothing loads
  Test2 = 42, // nothing loads
  Test3 = 43, // nothing loads
  Test4 = 44, // nothing loads
}


#[derive(Debug)]
pub enum GameMode {
  Normal = 0,
  Tutorial = 1,
  TutorialB = 2,
  Ending = 3,
  Load = 4,
}

impl TryFrom<i32> for GameMode {
    type Error = ();

    fn try_from(value: i32) -> Result<Self, Self::Error> {
      match value {
        0 => Ok(Self::Normal),
        1 => Ok(Self::Tutorial),
        2 => Ok(Self::TutorialB),
        3 => Ok(Self::Ending),
        4 => Ok(Self::Load),
        _ => panic!("unrecognized gamemode")
      }
    }
}
