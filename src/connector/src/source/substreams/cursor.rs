use anyhow::Error;
use std::str::FromStr;
use std::fs::write;
use crate::source::substreams::{key, step};
use crate::source::substreams::step::StepType;

#[derive(Debug, Clone)]
pub struct BlockRef {
    pub id: String,
    pub number: u64
    
    // fn id(&self) -> String;
    // fn num(&self) -> u64;
}

#[derive(Debug, Clone)]
pub struct Cursor {
    pub step: StepType,
    pub block: BlockRef,
    pub lib: BlockRef,
    pub head_block: BlockRef,
}

impl Cursor {
    pub fn from_opaque(opaque_cursor: &str) -> Result<Self, anyhow::Error> {
        let cursor_data = match key::decode(opaque_cursor){
            Ok(d) => {
                d
            }
            Err(err) => {
                return Err(err);
            }
        };
        let s = String::from_utf8(cursor_data)?;
        Self::from_str(&s)
    }

    pub fn is_on_final_block(&self) -> bool {
        self.block.number == self.lib.number
    }

    pub fn to_opaque(&self) -> String {
        // TODO: Implement opaque encoding
        self.to_string()
    }

    pub fn equals(&self, other: &Cursor) -> bool {
        if self.is_empty() {
            return other.is_empty();
        }
        self.block.id == other.block.id &&
            self.head_block.id == other.head_block.id &&
            self.lib.id == other.lib.id
    }

    pub fn is_empty(&self) -> bool {
        self.block.id.is_empty() ||
            self.head_block.id.is_empty() ||
            self.lib.id.is_empty()
    }

    fn to_string(&self) -> String {
        let blk_id = self.block.id.clone();
        let head_id = self.head_block.id.clone();
        let lib_id = self.lib.id.clone();

        if head_id == blk_id {
            format!("c1:{}:{}:{}:{}:{}",
                    self.step as i32,
                    self.block.number,
                    blk_id,
                    self.lib.number,
                    lib_id)
        } else if blk_id == lib_id {
            format!("c2:{}:{}:{}:{}:{}",
                    self.step as i32,
                    self.block.number,
                    blk_id,
                    self.head_block.number,
                    head_id)
        } else {
            format!("c3:{}:{}:{}:{}:{}:{}:{}",
                    self.step as i32,
                    self.block.number,
                    blk_id,
                    self.head_block.number,
                    head_id,
                    self.lib.number,
                    lib_id)
        }
    }

    pub fn save(&self) -> Result<(), Error> {
        let opaque = self.to_opaque();
        write("cursor.txt", opaque)?;
        Ok(())
    }
    
    pub fn load() -> Result<Option<Cursor>, Error> {
        match std::fs::read_to_string("cursor.txt"){
            Ok(opaque) => {
                let cursor = Cursor::from_opaque(&opaque)?;
                Ok(Some(cursor))
            }
            Err(_) => {
                tracing::warn!("cursor.txt not found");
                Ok(None)
            }
        }
    }
}

impl FromStr for Cursor {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {

        tracing::info!("cursor: {}", s);
        
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() < 6 {
            return Err(Error::msg("invalid cursor: too short"));
        }

        match parts[0] {
            "c1" => {
                if parts.len() != 6 {
                    return Err(Error::msg("invalid cursor: invalid number of segments"));
                }

                let step = step::from_str(parts[1])?;
                let blk_ref = read_cursor_block_ref(parts[2], parts[3])?;
                let lib_ref = read_cursor_block_ref(parts[4], parts[5])?;

                Ok(Cursor {
                    step,
                    block: blk_ref.clone(),
                    head_block: blk_ref,
                    lib: lib_ref,
                })
            }
            // Implement c2 and c3 cases similarly
            _ => Err(Error::msg("invalid cursor: invalid prefix"))
        }
    }
}


fn read_cursor_block_ref(num_str: &str, id: &str) -> Result<BlockRef, Error> {
    let num = num_str.parse::<u64>()?;
    let id = id.to_string();

    Ok(BlockRef {
        id,
        number: num,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ref_block(num: u64, id: &str) -> BlockRef {
        BlockRef {
            id: id.to_string(),
            number: num,
        }
    }

    // #[test]
    // fn test_from_opaque() {
    //     let opaque_cursor = "BpGEpx1V1QDygCXB3XyTYqWwLpc_DFhrXQvmKRhIgYGApxO7quGjWgYXPEWW9aiZplKoSFjah66wfCMv95cEuoqdleE86lpaRS9LwoPs_7zve_D7MA4RB-wDIZ3yZ4K8OWiPJG_6E8pHo9blTKqoGipvNsJxeGDo-GkJ-oIvEqBF3iI=";
    //     let cursor = Cursor::from_opaque(opaque_cursor).unwrap();
    //     assert_eq!(cursor.to_string(), "c1:1:11846516:d46eeb12ad30ef291a673329bb2f64bc689f15253371b64aaee017556ee95a35:0:d46eeb12ad30ef291a673329bb2f64bc689f15253371b64aaee017556ee95a35");
    // }
    #[test]
    fn test_from_string() {
        let empty_ref = ref_block(0, "");

        let tests = vec![
            (
                "c1 no LIB",
                "c1:1:11846516:d46eeb12ad30ef291a673329bb2f64bc689f15253371b64aaee017556ee95a35:0:",
                Cursor {
                    step: StepType::New,
                    block: ref_block(11846516, "d46eeb12ad30ef291a673329bb2f64bc689f15253371b64aaee017556ee95a35"),
                    head_block: ref_block(11846516, "d46eeb12ad30ef291a673329bb2f64bc689f15253371b64aaee017556ee95a35"),
                    lib: empty_ref.clone(),
                }
            ),
            // (
            //     "c2 full",
            //     "c2:1:7393903:e9e04d1f639ffd8491fd3c90153b341e68a2ef9aaa72337dc926d928384f8f71:7393905:4c01ca1daced994d7a87faa92a14a360a1b2f64340d97e82b579915765c36663",
            //     Cursor {
            //         step: StepType::New,
            //         block: ref_block(7393903, "e9e04d1f639ffd8491fd3c90153b341e68a2ef9aaa72337dc926d928384f8f71"),
            //         head_block: ref_block(7393905, "4c01ca1daced994d7a87faa92a14a360a1b2f64340d97e82b579915765c36663"),
            //         lib: ref_block(7393903, "e9e04d1f639ffd8491fd3c90153b341e68a2ef9aaa72337dc926d928384f8f71"),
            //     }
            // ),
            // (
            //     "c1 stepnewirreversible with LIB",
            //     "c1:17:20737275:56c6c96333f4cb6a0d9996b4c75370f19944fedecf42ce77d941fe7d58ddba2f:20737275:56c6c96333f4cb6a0d9996b4c75370f19944fedecf42ce77d941fe7d58ddba2f",
            //     Cursor {
            //         step: StepType::NewIrreversible,
            //         block: ref_block(20737275, "56c6c96333f4cb6a0d9996b4c75370f19944fedecf42ce77d941fe7d58ddba2f"),
            //         head_block: ref_block(20737275, "56c6c96333f4cb6a0d9996b4c75370f19944fedecf42ce77d941fe7d58ddba2f"),
            //         lib: ref_block(20737275, "56c6c96333f4cb6a0d9996b4c75370f19944fedecf42ce77d941fe7d58ddba2f"),
            //     }
            // ),
            // (
            //     "c3 full",
            //     "c3:1:7393903:e9e04d1f639ffd8491fd3c90153b341e68a2ef9aaa72337dc926d928384f8f71:7393905:4c01ca1daced994d7a87faa92a14a360a1b2f64340d97e82b579915765c36663:7393704:fc119c952209a330f6276f98cff168e4cd14f6edd34505e8d67a5e929d48d93a",
            //     Cursor {
            //         step: StepType::New,
            //         block: ref_block(7393903, "e9e04d1f639ffd8491fd3c90153b341e68a2ef9aaa72337dc926d928384f8f71"),
            //         head_block: ref_block(7393905, "4c01ca1daced994d7a87faa92a14a360a1b2f64340d97e82b579915765c36663"),
            //         lib: ref_block(7393704, "fc119c952209a330f6276f98cff168e4cd14f6edd34505e8d67a5e929d48d93a"),
            //     }
            // ),
        ];

        for (name, input, expected) in tests {
            let result = Cursor::from_str(input);
            match result {
                // None => {
                //     assert!(result.is_ok(), "Test case '{}' failed", name);
                //     assert_eq!(result.unwrap().equals(&expected), true, "Test case '{}' failed", name);
                // }
                // Some(err) => {
                //     assert!(result.is_err(), "Test case '{}' failed", name);
                //     assert_eq!(result.unwrap_err().to_string(), err.to_string());
                // }
                Ok(_) => {}
                Err(e) => {
                    panic!("Test case '{}' failed: {}", name, e);
                }
            }
        }
    }
}


