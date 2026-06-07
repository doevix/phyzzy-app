use phyzzy_rs::{
    self,
    Boundary,
    Loader,
    Mass,
    MassActuatorType,
    MassActuatorDataType,
    Model,
    Spring,
    SpringActuatorType,
    SpringActuatorDataType,
    V2D,
    World,
    WorldConfig
};
use std::{fs, path::PathBuf};


pub enum PhyzzyLoaderError {
    JSONReaderError,
    FileReaderError,
    ModelBuildError,
}

pub struct PhyzzyMeta {
    pub name: String,
    pub creator: String,
    pub created: String,
}
pub struct PhyzzySimElements {
    pub meta: PhyzzyMeta,
    pub model: Model,
    pub world: World,
    pub world_config: WorldConfig,
}

pub struct PhyzzyIO;
impl PhyzzyIO {
    pub fn import(file_name: &String, dt: f64) -> Result<PhyzzySimElements, PhyzzyLoaderError> {
        let file_txt_result = fs::read_to_string(file_name);
        let file_txt = match file_txt_result {
            Ok(json_str) => json_str,
            Err(_) => return Err(PhyzzyLoaderError::FileReaderError),
        };
        let output_result = match Loader::load_from_json_str(&file_txt) {
            Ok(loaded_data) => {
                let mut elements = PhyzzySimElements {
                    meta: PhyzzyMeta {
                        name: loaded_data.meta.name,
                        creator: loaded_data.meta.creator,
                        created: loaded_data.meta.created,
                    },
                    model: Model::new(loaded_data.model.wave_speed, loaded_data.model.wave_amplitude),
                    world: World::new(&V2D::new(loaded_data.world.area_sz.x, loaded_data.world.area_sz.y)),
                    world_config: WorldConfig {
                        gravity: V2D::new(loaded_data.world_config.gravity.x, loaded_data.world_config.gravity.y),
                        drag: loaded_data.world_config.drag,
                    },
                };

                elements.model.angle = loaded_data.model.angle;

                for mass in loaded_data.model.masses {
                    let pos = V2D::new(mass.pos.x, mass.pos.y);
                    let vel = V2D::new(mass.vel.x, mass.vel.y);
                    let pos_prv = pos - vel * dt;
                    let loaded_mass = Mass::load(mass.mass, mass.radius, &pos, &pos_prv);
                    elements.model.new_mass(loaded_mass);
                }

                for spring in loaded_data.model.springs {
                    let loaded_spring = Spring::new(spring.restlength, spring.springing, spring.dampening, spring.m_a, spring.m_b);
                    let spring_insert_result = elements.model.new_spring(loaded_spring);
                    match spring_insert_result {
                        Ok(_) => {},
                        Err(_) => { return Err(PhyzzyLoaderError::ModelBuildError) },
                    }
                }

                for muscle in loaded_data.model.muscles {
                    let muscle_type = match muscle.muscle_type {
                        SpringActuatorDataType::Classic => SpringActuatorType::ClassicMuscle,
                        SpringActuatorDataType::Relaxation => SpringActuatorType::RelaxationMuscle,
                    };
                    elements.model.new_muscle(muscle_type, muscle.spring, muscle.phase, muscle.sense);
                }

                for bladder in loaded_data.model.bladders {
                    let bladder_type = match bladder.bladder_type {
                        MassActuatorDataType::Balloon => MassActuatorType::Balloon,
                        MassActuatorDataType::Tank => MassActuatorType::Tank,
                    };
                    elements.model.new_bladder(bladder_type, bladder.mass, bladder.phase, bladder.sense, bladder.multiplier);
                }

                for (idx, layer) in loaded_data.model.collision_layers.iter().enumerate() {
                    elements.model.new_collision_layer();
                    for mass in &layer.masses {
                        elements.model.mass_to_collision_layer(idx, *mass);
                    }
                    for spring in &layer.springs {
                        elements.model.spring_to_collision_layer(idx, *spring);
                    }
                }

                for bound in loaded_data.world.bounds {
                    let pos = V2D::new(bound.pos.x, bound.pos.y);
                    let nrm = V2D::new(bound.nrm.x, bound.nrm.y);
                    let loaded_bound = Boundary::new(pos, nrm, bound.refl, bound.mu_s, bound.mu_k);
                    elements.world.bounds.push(loaded_bound);
                }
                Ok(elements)
            },
            Err(_) => Err(PhyzzyLoaderError::JSONReaderError),
        };
        output_result
    }
}
