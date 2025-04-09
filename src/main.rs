
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
}

fn get_file_name(url: &str, output_dir: &str) -> String {
    let output_file = url.split("/").collect::<Vec<&str>>();
    let ln = output_file.len();
    return String::from(output_dir.to_owned() + output_file[ln - 1]);
}

async fn download_file(url: &str, to: &str) -> Result<(), Error> {
    let res = reqwest::get(url).await?;
    let mut file = fs::File::create(to).expect("Could not create file");
    let content = res.bytes().await?;
    let mut pos = 0;
    while pos < content.len() {
        let bytes_written = file.write(&content[pos..]).expect("Could not write to file");
        pos += bytes_written;
    }
    Ok(())
}

fn update_wallpapers(images: &Vec<String>, fallback: &str) {
    let mut new_vec = Vec::new();
    for i in images.iter() {
        if i != "" {
            println!("WALL {}", i);
            new_vec.push(i.clone());
        }
    }
    use more_wallpapers::Mode;
    let _ = more_wallpapers::set_wallpapers_from_vec(new_vec, String::from(fallback), Mode::Crop);
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
    let fallback : String;
    //println!("{:?}", path);
    if file.exists() {
        println!("Loading settings from {}", settings_path);
        let contents = fs::read_to_string(settings_path)
                .expect("Should have been able to read the file")
                .to_string();
        let table = contents.parse::<Table>().unwrap();
        //println!("{}", table["links"]);
        let links_step_1 = table["links"].as_str();
        let links_step_2;
        match links_step_1 {
            Some(x) => links_step_2 = x.split(","),
            _none => return
        }
        let fallback_option = table["fallbacks"].as_str();
        fallback = match fallback_option {
            Some(x) => String::from(x),
            _none => String::from("")
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
            local_url.push(String::from(fallback.as_str()));
        }
    } else {
        println!("walltaker.toml doesn't exist, creating...");
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
        let test_url = update_link(links[i].as_str(), web_url[i].as_str()).await;
        if test_url != web_url[i] {
            println!("{}", test_url);
            web_url[i] = test_url;
            let file_name = get_file_name(web_url[i].as_str(), output_dir.as_str());
            local_url[i] = file_name;
            let download = download_file(web_url[i].as_str(), local_url[i].as_str()).await;
            println!("{}", local_url[i]);
            match download {
                Ok(_) => update_wallpapers(&local_url, fallback.as_str()),
                Err(e) => println!("{:?}", e)
            }
        }
        thread::sleep(sleep_time);
    }
}