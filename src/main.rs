
use std::env;
use std::fs;
use std::io::Write;
use reqwest;
use toml::Table;

use std::time::Duration;
use std::thread;

use reqwest::Error;
use json::parse;

fn parse_env_var(res: &str) -> String {
    if res.ends_with("/") {
        return String::from(res);
    } else {
        return String::from(res.to_owned() + "/");
    }
}

async fn update_link(id: &str, old: &str) -> String {
    async fn helper(id: &str, _old: &str) -> Result<String, Error> {
        let res = 
            reqwest::get("https://walltaker.joi.how/api/links/".to_owned()+id+".json")
                .await?.text().await?;
        return Ok(res);
    }

    let out = helper(id, old).await;
    match out {
        Ok(json) => {
            let parsed = parse(json.as_str()).unwrap();
            let mut output_str : String = json::stringify(parsed["post_url"].clone());
            output_str.pop();
            output_str.remove(0);
            return output_str;
        },
        Err(_e) => return String::from(old)
    }
    //let step_2 = match res.error_for_status {
    //    
    //}
        //?.text().await?;
    /*
    let parsed = parse(res.as_str()).unwrap();
    let mut output_str = json::stringify(parsed["post_url"].clone());
    output_str.pop();
    output_str.remove(0);
    return Ok(output_str);
    */
    
    //return String::from(old);
}

#[tokio::main]
async fn main() {
    //TODO: test if this even works
    let xdg_config_home : Result<String, env::VarError> = env::var("XDG_CONFIG_HOME");
    let user_home : Result<String, env::VarError> = env::var("HOME");
    let home : String;
    let path : String;
    //TODO: toml
    //println!("{:?}", xdg_config_home);
    home = match user_home {
        Ok(res) => res,
        Err(_) => String::from("~")
    };
    path = match xdg_config_home {
        Ok(res) =>  parse_env_var(&res),
        Err(_) => String::from(home + "/.config/")
    };
    let settings_path = path.to_owned() + "walltaker.toml";
    let file = std::path::Path::new(&settings_path);
    let mut links = vec![String::from("")];
    let mut web_url = vec![String::from("")];
    let mut local_url = vec![String::from("")];
    let output_dir : String;
    //println!("{:?}", path);
    if file.exists() {
        println!("Loading settings from {}", settings_path);
        let contents = fs::read_to_string(settings_path)
                .expect("Should have been able to read the file")
                .to_string();
        let table = contents.parse::<Table>().unwrap();
        println!("{}", table["links"]);
        let links_step_1 = table["links"].as_str();
        let links_step_2;
        match links_step_1 {
            Some(x) => links_step_2 = x.split(","),
            _none => return
        }
        let fallback_option = table["fallbacks"].as_str();
        let fallback = match fallback_option {
            Some(x) => x,
            _none => ""
        };
        let output_option = table["output_dir"].as_str();
        output_dir = match output_option {
            Some(x) => parse_env_var(x),
            _none => String::from("/tmp")
        };
        for i in links_step_2 {
            println!("{}", i);
            links.push(String::from(i));
            web_url.push(String::from(""));
            local_url.push(String::from(fallback));
        }
    } else {
        println!("settings.toml doesn't exist, creating...");
        //TODO: terminal setup
        fn write_file(settings_path : &String) -> std::io::Result<()> {
            println!("write file {}", settings_path);
            let mut buffer = fs::File::create(settings_path)?;
            buffer.write_all(b"links = \"1,2\"\noutput_dir=\"/tmp\"\nfallback=\"/usr/share/desktop-base/emerald-theme/wallpaper/contents/images/1920x1080.svg\"")?;
            Ok(())
        }
        let _ = write_file(&settings_path);
        println!("Edit {} to set up walltaker.", settings_path);
        return;
    }
    let sleep_time = Duration::from_secs(20);
    let mut i = 1;
    loop {
        i += 1;
        if i >= links.len() {
            i = 1;
        }
        let test_url = update_link(links[i].as_str(), web_url[i].as_str());

        thread::sleep(sleep_time);
    }
}

/* old code
const FALLBACK: &str = "../usr/share/desktop-base/homeworld-theme/wallpaper/contents/images/1920x1080.svg";
const FOLDER: &str = "/tmp/";
use std::thread;
use std::time::Duration;
use std::io::{Write};
use std::fs;
use reqwest::Error;
use json::parse;

fn update_wallpapers(images: &Vec<String>) {
    let mut new_vec = Vec::new();
    for i in images.iter() {
        if i != "" {
            println!("{}", i);
            new_vec.push("/tmp/".to_owned() + i.as_str());
        }
    }
    use more_wallpapers::Mode;
    let _ = more_wallpapers::set_wallpapers_from_vec(new_vec, FALLBACK.to_string(), Mode::Crop);
}

async fn download_file(link: &str) -> Result<&str, Error> {
    let res = reqwest::get(link).await?;
    let output_file = link.split("/").collect::<Vec<&str>>();
    let ln = output_file.len();
    println!("{:?}", FOLDER.to_owned() + output_file[ln - 1]);
    let mut file = fs::File::create(FOLDER.to_owned() + output_file[ln - 1]).expect("help");
    let content = res.bytes().await?;
    let mut pos = 0;
    while pos < content.len() {
        let bytes_written = file.write(&content[pos..]).expect("im in hell");
        pos += bytes_written;
    }
    Ok(output_file[ln - 1])
}

async fn update_link(link: &String) -> Result<String, Error> {
    let res = reqwest::get("https://walltaker.joi.how/api/links/".to_owned()+link+".json").await?.text().await?;
    let parsed = parse(res.as_str()).unwrap();
    let mut output_str = json::stringify(parsed["post_url"].clone());
    output_str.pop();
    output_str.remove(0);
    return Ok(output_str);
}

#[tokio::main]
async fn main() {
    let path = std::path::Path::new("links.txt");
    let mut links = vec![String::from("just a string :)")];
    let mut url = vec![String::from("")];
    let mut download_url = vec![String::from("")];
    if path.exists() {
        let contents = fs::read_to_string("links.txt")
            .expect("Should have been able to read the file")
            .to_string();
        //TODO:
        let split_part_one = contents.split("=").collect::<Vec<&str>>();
        let split_part_two = split_part_one[1].split(",").collect::<Vec<&str>>();
        for i in split_part_two.into_iter() {
            let j = i.to_string();
            links.push(j.clone());
            url.push(String::from(""));
            download_url.push(String::from(FALLBACK));
        }
    } else {
        println!("links.txt not found");
        fn write_file() -> std::io::Result<()> {
            let mut buffer = fs::File::create("links.txt")?;
            buffer.write_all(b"links=1,2")?;
            Ok(())
        }
        let _ = write_file();
        println!("Created links.txt, edit the file to use your links.");
        return;
    };
    let sleep_time = Duration::from_secs(20);
    let mut i = 1;
    update_wallpapers(&download_url);
    loop {
        thread::sleep(sleep_time);
        i += 1;
        if i >= links.len() {
            i = 1;
        }
        let temp = update_link(&links[i]).await.expect("what");
        if temp.clone() != url[i] {
            let clone = temp.clone();
            let fileurl = download_file(&clone).await.expect("file error");
            url[i] = temp;
            download_url[i] = (&fileurl).to_string();
            
            update_wallpapers(&download_url);
        }
    }
}
*/