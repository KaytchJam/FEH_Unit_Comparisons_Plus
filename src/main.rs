#[macro_use] extern crate rocket;
extern crate nalgebra as na;

use feh_manager::{FehKDTree, FehManager};
use rocket::fs::NamedFile;
use rocket::State;

mod lerp;
mod kdtree;
mod feh_manager;

#[get("/")]
async fn index() -> Option<NamedFile> {
    return NamedFile::open("public/docs/index.html").await.ok();
}


#[get("/public/<pub_file>/<file_name>")]
async fn get_pub_file(pub_file: &str, file_name: &str) -> Option<NamedFile> {
    println!("The path is: public/{}/{}", pub_file, file_name);
    
    match pub_file {
        "scripts" => {
            if file_name[(file_name.len() - 3)..file_name.len()] != *".js" {
                return None;
            }
        },

        "styles" => {
            if file_name[(file_name.len() - 4)..file_name.len()] != *".css" {
                return None;
            }
        },
        
        "docs" => {
            if file_name[(file_name.len() - 5)..file_name.len()] != *".html" {
                return None;
            }
        },
        _ => {
            return None;
        }
    };

    return NamedFile::open("public".to_owned() + "/" + pub_file + "/" + file_name).await.ok();
}

#[get("/<pub_file>/<file_name>")]
async fn get_file(pub_file: &str, file_name: &str) -> Option<NamedFile> {
    return get_pub_file(pub_file, file_name).await;
}

#[get("/world")]
fn world() -> &'static str {
    "Hello, world!"
}

#[get("/?lerp&<startunit>&<endunit>")]
fn lerp_units<'query, 'storage>(startunit: &'query str, endunit: &'query str, unit_pack_state: &'storage State<FehManager>, tree_state: &'storage State<FehKDTree>) -> String {
    let inputs: [&str; 2] = [startunit, endunit];

    for unit_input in inputs.into_iter() {
        println!("input -> {}", unit_input);
        if !unit_pack_state.contains(unit_input) {
            println!("Not contained!");
            return "[]".to_owned();
        }
    }
    
    return unit_pack_state.lerp_units_with_dist(startunit, endunit, &tree_state).to_json_names();
}

#[get("/all_units")]
fn  all_units(unit_pack_state: &State<FehManager>) -> String {
    return unit_pack_state.all_units();
}


#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    let fm: FehManager = FehManager::init("./data/FEH_Unit_List.csv").unwrap();
    let ft: FehKDTree = FehKDTree::construct_kdtree(&fm);

    let _rocket: rocket::Rocket<rocket::Ignite> = rocket::build()
        .mount("/", routes![index, world, get_pub_file, get_file, all_units, lerp_units])
        .manage(fm)
        .manage(ft)
        .launch()
        .await?;

    Ok(())
}
