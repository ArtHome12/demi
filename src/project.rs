/* ===============================================================================
Simulation of the evolution of the animal world.
Project TOML file read/save/parse.
28 Feb 2021.
----------------------------------------------------------------------------
Licensed under the terms of the GPL version 3.
http://www.gnu.org/licenses/gpl-3.0.html
Copyright (c) 2013-2022 by Artem Khomenko _mag12@yahoo.com.
=============================================================================== */

use std::{fs, collections::HashMap, };
use std::path::PathBuf;
use serde::Deserialize;

use crate::geom::*;
use crate::reactions::*;


// Content of a toml project file
#[derive(Deserialize)]
struct Toml {
   pub width: usize,
   pub height_ratio: f32,
   resolution: f32,
   max_animal_stack: usize,
   heterotroph_color: String,
   elements: Vec<ElementAttributes>,
   chemical: Vec<ReactionAttributes>,
   colors: HashMap<String, Colors>,
   luca: LucaAttributes,
}

impl Toml {
   pub fn new(filename: &PathBuf) -> Self {
      let data = fs::read_to_string(filename).expect("Unable to read project file");
      toml::from_str(&data).unwrap()
   }
}

#[derive(Deserialize)]
struct ElementAttributes {
   name: String,
   color: String,
   volatility: f32,
   amount: usize,
}


#[derive(Debug, Deserialize)]
struct ReactionReagent {
   element: String,
   amount: usize,
}

#[derive(Deserialize)]
struct ReactionAttributes {
   name: String,
   vitality: usize,
   color: String,
   left: Vec<ReactionReagent>,
   right: Vec<ReactionReagent>,
}

// Section of the toml file withcolors
type Colors = (u8, u8, u8);

#[derive(Deserialize, Debug, Clone)]
pub struct LucaAttributes {
   pub digestion: String,
}


#[derive(Debug, Clone)]
pub struct Project {
   pub size: Size,
   pub resolution: f32,
   pub max_animal_stack: usize,
   pub heterotroph_color: iced::Color,
   pub elements: Vec<Element>,
   pub reactions: Reactions,
   pub ui_reactions: UIReactions,
   pub luca_reaction: usize, // first organism
   pub filename: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Element {
   pub name: String,
   pub color: iced::Color,
   pub volatility: f32,
   pub init_amount: usize,
}

impl Project {
   pub fn new(filename: PathBuf) -> Self {

      // Reads reactions reagents
      fn do_reagents(elements: &Vec<Element>, part: &Vec<ReactionReagent>) -> Vec<Reagent> {
         part.iter().map(|reagent| {

            // Element index from its name
            let index = elements.iter()
            .position(|v| v.name == reagent.element)
            .expect(&format!("Unknown chemical reagent {}", reagent.element));

            Reagent {
               index,
               amount: reagent.amount,
            }
         }).collect()
      }

      // Read general data from toml file
      let toml = Toml::new(&filename);
      let width = toml.width;
      let size = Size::new(width, (width as f32 * toml.height_ratio) as usize);

      // Map elements from Hash to Vec with colors in internal representation
      let elements: Vec<Element> = toml.elements.iter().map(|val| {

         // Color item from hash by name
         let color = Project::color_by_name(&toml, &val.color);

         Element {
            name: val.name.clone(),
            volatility: val.volatility,
            color,
            init_amount: val.amount,
         }
      }).collect();

      // Read reactions twice, for model and for UI
      let reactions = toml.chemical.iter().map(|val| {
         Reaction {
            vitality: val.vitality,
            left: do_reagents(&elements, &val.left),
            right: do_reagents(&elements, &val.right),
         }
      }).collect::<Reactions>();

      let ui_reactions = toml.chemical.iter().map(|val| {
         UIReaction {
            name: val.name.to_owned(),
            color: Project::color_by_name(&toml, &val.color),
         }
      }).collect::<UIReactions>();

      let heterotroph_color = Project::color_by_name(&toml, &toml.heterotroph_color);

      // At start all elements should be visible, collect its indexes
      // let len = toml.elements.len();
      // let vis_elem_indexes = vec![true; len];
      // let vis_reac_indexes = vec![true; len];

      // Check data for first organism
      let reaction_name = &toml.luca.digestion;
      let luca_reaction = ui_reactions.index(reaction_name)
      .expect(&format!("Unknown reaction for digestion LUCA {}", reaction_name));

      Self {
         size,
         resolution: toml.resolution,
         max_animal_stack: toml.max_animal_stack,
         heterotroph_color,
         elements,
         reactions,
         ui_reactions,
         luca_reaction,
         filename,
         // vis_elem_indexes,
         // vis_reac_indexes,
         // vis_dead: true,
      }
   }

   fn color_by_name(toml: &Toml, color: &String) -> iced::Color {
      // Color from r, g, b array or default
      let color = toml.colors.get(color);
      if let Some(color) = color {
         iced::Color::from_rgb8(color.0, color.1, color.2)
      } else {
         iced::Color::BLACK
      }
   }
}
