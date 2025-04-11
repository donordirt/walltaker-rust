
use std::env;
use std::fs;
use std::io;
use std::io::Write;
use reqwest;
use toml::Table;

use std::time::Duration;
use std::thread;

use reqwest::Error;
use json::parse;

fn fix_path(res: &str) -> String {
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
    let xdg_config_home : Result<String, env::VarError> = env::var("XDG_CONFIG_HOME");
    let user_home : Result<String, env::VarError> = env::var("HOME");
    let home : String;
    let path : String;
    home = match user_home {
        Ok(res) => res,
        Err(_) => String::from("~")
    };
    path = match xdg_config_home {
        Ok(res) =>  fix_path(&res),
        Err(_) => String::from(home + "/.config/")
    };
    let settings_path = path.to_owned() + "walltaker.toml";
    let file = std::path::Path::new(&settings_path);
    let mut links = vec![String::from("")];
    let mut web_url = vec![String::from("")];
    let mut local_url = vec![String::from("")];
    let output_dir : String;
    let fallback : String;
    if file.exists() {
        println!("Loading settings from {}", settings_path);
        let contents = fs::read_to_string(settings_path)
                .expect("Should have been able to read the file")
                .to_string();
        let table = contents.parse::<Table>().unwrap();
        let links_step_1 = table["links"].as_str();
        let links_step_2;
        match links_step_1 {
            Some(x) => links_step_2 = x.split(","),
            _none => return
        }
        let fallback_option = table["fallback"].as_str();
        fallback = match fallback_option {
            Some(x) => String::from(x),
            _none => String::from("")
        };
        let output_option = table["output_dir"].as_str();
        output_dir = match output_option {
            Some(x) => fix_path(x),
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
        println!("You can edit {} later to modify these settings.", settings_path);
        let mut settings_str: String = String::from("links=\"");
        
        let mut adding_links : bool = true;
        let mut first_link : bool = true;
        let mut second_link : bool = true;
        let stdin = io::stdin();
        while adding_links {
            if !first_link && second_link {
                println!("Press enter without typing anything to continue.");
                second_link = false;
            } else if first_link {
                println!("Enter your links, one at a time: ");
            }
            let mut link_in = String::new();
            stdin.read_line(&mut link_in).expect("Failed to read stdin");
            link_in.pop();
            if link_in == "" && !first_link {
                adding_links = false;
            } else {
                //there's probably an easier way to do this but it works
                if first_link {
                    settings_str = settings_str + link_in.as_str();
                    first_link = false;
                } else {
                    settings_str = settings_str + "," + link_in.as_str();
                }
                links.push(link_in);
                web_url.push(String::from(""));
            }
        }
        
        println!("\nSpecify where files should be written to: (defaults to /tmp/ if you don't enter anything)");
        let mut output_in : String = String::new();
        stdin.read_line(&mut output_in).expect("Failed to read stdin");
        output_in.pop();
        let mut using_temp = false;
        if output_in == "" {
            output_in = String::from("/tmp/");
            using_temp = true;
        } else if fix_path(output_in.as_str()) == "/tmp/" {
            using_temp = true;
        }
        settings_str = settings_str + "\"\noutput_dir=\"" + output_in.as_str();
        output_dir = output_in;

        println!("\nSpecify a fallback wallpaper.");
        println!("This wallpaper will be used when starting your computer and on extra monitors.");
        if !using_temp {
            println!("If you specified a directory outside of /tmp/ before, you can ignore this.");
        }
        println!("Enter path:");
        let mut fallback_in : String = String::new();
        stdin.read_line(&mut fallback_in).expect("Failed to read stdin");
        settings_str = settings_str + "\"\nfallback=\"" + fallback_in.as_str() + "\"";
        fallback_in.pop();
        for i in &links {
            if i != "" {
                local_url.push(String::from(fallback_in.as_str()));
            }
        }
        fallback = fallback_in;

        fn write_file(settings_path : &String, settings_str: &str) -> std::io::Result<()> {
            println!("\nWrote file {}", settings_path);
            let mut buffer = fs::File::create(settings_path)?;
            buffer.write_all(settings_str.as_bytes())?;
            Ok(())
        }
        let _ = write_file(&settings_path, settings_str.as_str());
        println!("I'd recommend adding this file as a startup script.");
        thread::sleep(Duration::from_secs(5));
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