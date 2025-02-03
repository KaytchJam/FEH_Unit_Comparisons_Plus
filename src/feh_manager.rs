extern crate nalgebra as na;
use std::fs;
use std::collections::BTreeMap;
use std::ops::Index;

use crate::lerp::MonomialLerp;
use crate::kdtree::CKDTree;
use std::sync::Arc;

#[derive(Debug)]
struct FehUnit {
    m_name: String,
    m_character: String,
    m_stats: na::Vector5<f32>
}

impl FehUnit {
    fn new(name: String, character: String, stats: na::Vector5<f32>) -> Self {
        return FehUnit {
            m_name: name,
            m_character: character,
            m_stats: stats
        };
    }

    // Returns a reference to a FehUnit's stats, which is internally stored
    // as an na::Vector5<f32>
    fn get_stats(&self) -> &na::Vector5<f32> {
        return &self.m_stats;
    }

    fn get_name(&self) -> &str {
        return self.m_name.as_str();
    }

    fn get_character(&self) -> &str {
        return self.m_character.as_str();
    }

    fn describe(&self) -> () {
        println!("[\nUnit Name: {}\nCharacter: {}\nStats: {}", self.m_name, self.m_character, self.m_stats);
    }

    fn as_arc(self) -> Arc<Self> {
        return Arc::new(self);
    }
}

// for compatability w/ KDTREE
impl Index<usize> for FehUnit {
    type Output = f32;
    fn index(&self, index: usize) -> &Self::Output {
        return &self.m_stats[index];
    }
}

#[derive(Debug)]
pub struct FehManager {
    m_unit_map: BTreeMap<String, Arc<FehUnit>>,
    // m_unit_kdtree: Option<CKDTree<'a,f32,FehUnit>>
}

pub struct FehKDTree(CKDTree<Arc<FehUnit>>);
impl FehKDTree {
    pub fn construct_kdtree(man: &FehManager) -> Self {
        let mut unit_tree: CKDTree<Arc<FehUnit>> = CKDTree::new(5);
        for unit_data in man.m_unit_map.iter() {
            unit_tree.push_wrapped(unit_data.1.clone());
        }

        return FehKDTree(unit_tree);
    }
}

// impl Deref for FehKDTree {
//     type Target = CKDTree<Arc<FehUnit>>;
//     fn deref(&self) -> &Self::Target {
//         return &self.0;
//     }
// }

impl FehManager {
    fn populate_tuple<'temp_tuple, 'from_file>(stream_net: &'temp_tuple mut (&'from_file str, &'from_file str, na::Vector5<f32>), comma_slice: &'from_file str, comma_cnt: usize) {
        //println!("name: {}, char: {}, comma_slice: {}", stream_net.0, stream_net.1, comma_slice);
        match comma_cnt {
            2 => { stream_net.0 = comma_slice; },
            3 => { stream_net.1 = comma_slice; },
            30..=34 => {
                let actual_idx: usize = comma_cnt - 30;
                stream_net.2[actual_idx] = comma_slice.parse::<f32>().unwrap();
            },
            _ => ()
        };
    }

    fn populate_unit_map<'a,'b>(mut self, file_str: &'a String, mut stream_net: &'b mut (&'a str, &'a str, na::Vector5<f32>)) -> Self {
        let mut file_iter = file_str.as_bytes().iter();
        let mut offset = 0;
        while *file_iter.next().unwrap() != b'\n' { offset += 1; } // skip the first row
        offset += 1;

        let mut comma_count: usize = 0;
        let mut prev_comma_idx: usize;
        let mut cur_comma_idx: usize = offset;

        for (idx, character) in file_iter.enumerate() {
            match *character {
                b',' => {
                    comma_count += 1;
                    prev_comma_idx = cur_comma_idx;
                    cur_comma_idx = idx + offset;

                    let column_slice: &str = &file_str[(prev_comma_idx + 1)..cur_comma_idx];
                    FehManager::populate_tuple(&mut stream_net, column_slice, comma_count);
                },

                b'\n' => {
                    println!("row unit: {:?}", stream_net);
                    let name_slice: String = stream_net.0.to_string();
                    let new_unit: FehUnit = FehUnit::new(stream_net.0.to_string(), stream_net.1.to_string(), stream_net.2);
                    self.m_unit_map.insert(name_slice, Arc::new(new_unit));

                    comma_count = 0;
                },

                _ => ()
            };
        }

        return self;
    }

    pub fn init(fpath: &str) -> std::result::Result<FehManager, &'static str> {
        let man: FehManager = FehManager { m_unit_map: BTreeMap::new(), };

        // structures
        let mut stream_net: (&str, &str, na::Vector5<f32>) = ("", "", na::Vector5::zeros());
        let file_str = fs::read_to_string(fpath).expect("The path shouldn't be wrong here. Nothing abnormal about the file either.");
        return std::result::Result::Ok(man.populate_unit_map(&file_str, &mut stream_net));
    }

    pub fn num_units(&self) -> usize {
        return self.m_unit_map.len();
    }

    pub fn get_unit(&self, unit_name: &str) -> &FehUnit {
        return self.m_unit_map.get(unit_name).unwrap().as_ref();
    }

    pub fn all_units(&self) -> String {
        let mut json_list: String = String::from("[");

        let mut unit_iter = self.m_unit_map.iter();
        let lead = unit_iter.next().unwrap().1.m_name.as_str();
        json_list = json_list + "\"" + lead + "\"";

        for pair in unit_iter {
            json_list = json_list + ",\"" + pair.1.m_name.as_str() + "\"";
        }

        json_list += "]";
        return json_list;
    }

    pub fn search(&self, query: &str) -> Option<&str> {
        return self.m_unit_map.get(query).and_then(|fu| Some(fu.m_name.as_str()));
    }

    pub fn contains(&self, query: &str) -> bool {
        return self.m_unit_map.contains_key(query);
    }

    fn mock_unit(stats: &na::Vector5<f32>) -> FehUnit {
        return FehUnit::new("".to_string(), "".to_string(), stats.clone());
    }

    fn vec5_squared_metric_distance(f1: &FehUnit, f2: &FehUnit) -> f32 {
        let mut squared_dist: f32 = 0f32;
        for i in 0..5 {
            squared_dist += (f1.m_stats[i] - f2.m_stats[i]).powf(2f32);
        }

        return squared_dist;
    }

    fn closest_to<'man, 'temp>(&'man self, point: &'temp na::Vector5<f32>, tree: &'man FehKDTree) -> &FehUnit {
        return tree.0.nearest_neighbor(&FehManager::mock_unit(point), |f1,f2| FehManager::vec5_squared_metric_distance(f1, f2)).unwrap();
    }

    pub fn lerp_units<'man>(&'man self, unit1: &str, unit2: &str, tree: &'man FehKDTree) -> FehVec {
        let start_unit: &FehUnit = self.m_unit_map.get(unit1).unwrap();
        let end_unit: &FehUnit = self.m_unit_map.get(unit2).unwrap();

        let mut lerp_units: Vec<&FehUnit> = Vec::new();
        for vec in MonomialLerp::quick_iter(1f32, start_unit.get_stats(), end_unit.get_stats(), 10) {
            lerp_units.push(self.closest_to(&vec, tree));
        }
        
        return FehVec(lerp_units);
    }

    pub fn lerp_units_with_dist<'man>(&'man self, unit1: &str, unit2: &str, tree: &'man FehKDTree) -> FehVecPlus {
        let start_unit: &FehUnit = self.m_unit_map.get(unit1).unwrap();
        let end_unit: &FehUnit = self.m_unit_map.get(unit2).unwrap();

        let mut lerped_units : Vec<(&FehUnit, f32)> = Vec::new();
        for lerp_point in MonomialLerp::quick_iter(1f32, start_unit.get_stats(), end_unit.get_stats(), 10) {
            let nearest: &FehUnit = self.closest_to(&lerp_point, tree);
            let distance: f32 = nearest.get_stats().metric_distance(&lerp_point);
            lerped_units.push((nearest, distance));
        }

        lerped_units.push((self.get_unit(unit2), 0f32));
        return FehVecPlus(lerped_units);
    }
}

#[derive(Debug)]
pub struct FehVec<'man>(Vec<&'man FehUnit>);

#[derive(Debug)]
pub struct FehVecPlus<'man>(Vec<(&'man FehUnit, f32)>);

impl<'man> FehVec<'man> {
    fn json_names<I: Iterator<Item = &'man FehUnit>>(mut iter: I) -> String {
        let mut json_list: String = String::from("[");
        json_list = json_list + "\"" + iter.next().unwrap().get_name() + "\"";
        for unit in iter {
            json_list = json_list + ",\"" + unit.get_name() + "\"";
        }

        return json_list + "]";
    }

    pub fn to_json_names(self) -> String {
        let mut iter = self.0.into_iter();
        let mut json_list: String = String::from("[");
        json_list = json_list + "\"" + iter.next().unwrap().get_name() + "\"";

        for unit in iter {
            json_list = json_list + ",\"" + unit.get_name() + "\"";
        }

        return json_list + "]";
    }
}

impl<'man> FehVecPlus<'man> {
    fn format_unit_distance_tuple(unit_and_distance: (&FehUnit, f32)) -> String {
        let mut stringified = String::from("[");
        stringified += "\"";
        stringified += unit_and_distance.0.get_name();
        stringified += "\", ";
        stringified += unit_and_distance.1.to_string().as_str();
        return stringified + "]";
    }

    pub fn to_json_names(self) -> String {
        let mut iter = self.0.into_iter();
        let mut json_list: String = String::from("[");
        
        json_list += &Self::format_unit_distance_tuple(iter.next().unwrap());
        for unit_dist_pair in iter {
            json_list += ",";
            json_list += &Self::format_unit_distance_tuple(unit_dist_pair);
        }

        return json_list + "]";
    }
}

#[cfg(test)]
mod tests {
    use super::{CKDTree, FehUnit};

  #[test]
  fn kdtree_test() {
    // Making up feh units, testing KDTree

    let my_unit = FehUnit {
        m_name: "Dragonlord Tiki".to_owned(),
        m_character: "Tiki".to_owned(),
        m_stats: na::Vector5::new(100f32, 55f32, 65f32, 55f32, 45f32)
    }.as_arc();

    let unit_two = FehUnit {
        m_name: "King Hector".to_owned(),
        m_character: "Hector".to_owned(),
        m_stats: na::Vector5::new(150f32, 80f32, 5f32, 70f32, 55f32)
    }.as_arc();

    let unit_three = FehUnit {
        m_name: "Queen Camilla".to_owned(),
        m_character: "Camilla".to_owned(),
        m_stats: na::Vector5::new(85f32, 95f32, 80f32, 40f32, 40f32)
    }.as_arc();

    let unit_four = FehUnit {
        m_name: "Goddess Loki".to_owned(),
        m_character: "Loki".to_owned(),
        m_stats: na::Vector5::new(70f32, 180f32, 100f32, 20f32, 50f32)
    }.as_arc();

    let unit_five = FehUnit::new("Sable Camus".to_owned(), "Camus".to_owned(), na::Vector5::new(75f32, 200f32, 70f32, 50f32, 10f32)).as_arc();
    let unit_six = FehUnit::new("Copper Lukas".to_owned(), "Lukas".to_owned(), na::Vector5::new(100f32, 60f32, 15f32, 80f32, 20f32)).as_arc();
    let unit_seven = FehUnit::new("Seer Shulk".to_owned(), "Shulk".to_owned(), na::Vector5::new(55f32, 90f32, 60f32, 35f32, 30f32)).as_arc();

    let mock_unit = FehUnit::new("Summoner".to_owned(), "Summoner".to_owned(), na::Vector5::new(55f32, 75f32, 90f32, 30f32, 80f32));
    let mut feh_tree = CKDTree::new(5);
    feh_tree.push_wrapped(unit_two);
    feh_tree.push_wrapped(unit_three);
    feh_tree.push_wrapped(unit_four);
    feh_tree.push_wrapped(unit_five);
    feh_tree.push_wrapped(unit_six);
    feh_tree.push_wrapped(unit_seven);

    let closest = feh_tree.nearest_neighbor(&mock_unit, |u1,u2| u1.m_stats.metric_distance(&u2.m_stats)).unwrap();

    println!("FEH TREE: {:?}", feh_tree);
    println!("CLOSEST UNIT: {:?}", closest);
  }
}