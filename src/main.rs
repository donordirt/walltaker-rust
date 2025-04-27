
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

fn prompt_links() -> Vec<String> {
    let mut res: Vec<String> = Vec::new();
    let stdin = io::stdin();
    let mut adding_links : bool = true;
    let mut first_link : bool = true;
    let mut second_link : bool = true;
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
            let valid_numbers = vec!["0","1","2","3","4","5","6","7","8","9"];
            let mut valid_link = true;
            for char in link_in.chars() {
                if !valid_numbers.contains(&char.to_string().as_str()) {
                    println!("Invalid character {} in link {}", char, link_in);
                    valid_link = false;
                    break;
                }
            }
            if valid_link {
                res.push(link_in);
                if first_link {
                    first_link = false;
                }
            }
        }
    }
    return res;
}

fn prompt_output_dir() -> String {
    let stdin = io::stdin();
    println!("\nSpecify where files should be written to: (defaults to /tmp/ if you don't enter anything)");
    let mut output_in : String = String::new();
    stdin.read_line(&mut output_in).expect("Failed to read stdin");
    output_in.pop();
    if output_in == "" {
        output_in = String::from("/tmp/");
    }
    return fix_path(output_in.as_str());
}

fn prompt_fallback(using_temp: bool) -> String {
    let stdin = io::stdin();
    println!("\nSpecify a fallback wallpaper.");
    println!("This wallpaper will be used when starting your computer and on extra monitors.");
    if !using_temp {
        println!("If you specified a directory outside of /tmp/ before, you can probably ignore this.");
    }
    println!("Enter path:");
    let mut fallback_in : String = String::new();
    stdin.read_line(&mut fallback_in).expect("Failed to read stdin");
    fallback_in.pop();
    return String::from(fallback_in);
}

fn create_settings_file(
    write_to: &str,
    links: &str,
    output_dir: &str,
    fallback: &str
) -> std::io::Result<()> {
    let output_str: String = String::from("links=\"") +
        links + "\"\noutput_dir=\"" + 
        output_dir + "\"\nfallback=\""+
        fallback + "\"";
    let mut buffer = fs::File::create(write_to)?;
    buffer.write_all(output_str.as_bytes())?;
    println!("\nWrote file {}", write_to);
    return Ok(());
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
            let parsed_result = parse(json.as_str());
            match parsed_result {
                Ok(parsed) => {
                    let mut output_str : String = json::stringify(parsed["post_url"].clone());
                    output_str.pop();
                    output_str.remove(0);
                    return output_str;
                },
                Err(_) => {
                    println!("Failed to get JSON from walltaker for link {}",id);
                    return String::from(old)
                }
            }
        },
        Err(_) => return String::from(old)
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
    let args: Vec<String> = env::args().collect();
    let mut args_iter = args.into_iter();
    let fix_option = args_iter.find(|x| x == "--fix");
    let fix = match fix_option {
        Some(_) => true,
        _ => false
    };

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
    let mut links: Vec<String> = Vec::new();
    let mut web_url: Vec<String> = Vec::new();
    let mut local_url: Vec<String> = Vec::new();
    let output_dir : String;
    let fallback : String;
    if file.exists() {
        println!("Loading settings from {}", settings_path);
        let contents = fs::read_to_string(&settings_path)
                .expect("Should have been able to read the file");
        let table_res = contents.parse::<Table>();
        match table_res {
            Ok(_) => {
                let table = table_res.unwrap();
                let mut modified_toml = false;
                let links_step_1 = table["links"].as_str();
                let links_step_2: Vec<String>;
                match links_step_1 {
                    Some(x) => {
                        if x != "" {
                            let old_vec = x.split(",");
                            let mut new_vec: Vec<String> = Vec::new();
                            for i in old_vec {
                                new_vec.push(String::from(i));
                            }
                            links_step_2 = new_vec;
                        } else if fix {
                            modified_toml = true;
                            links_step_2 = prompt_links();
                        } else {
                            return;
                        }
                    },
                    _none => {
                        if fix {
                            links_step_2 = prompt_links();
                        } else {
                            return;
                        }
                    }
                }
                let output_option = table["output_dir"].as_str();
                match output_option {
                    Some(x) => {
                        if x == "" {
                            if fix {
                                modified_toml = true;
                                output_dir = fix_path(prompt_output_dir().as_str());
                            } else {
                                println!("Failed to read output directory, defaulting to /tmp/");
                                modified_toml = true;
                                output_dir = fix_path("/tmp/");
                            }
                        } else {
                            output_dir = fix_path(x)
                        }
                    },
                    _none => {
                        if fix {
                            output_dir = prompt_output_dir();
                            modified_toml = true;
                        } else {
                            println!("Failed to read output directory, defaulting to /tmp/");
                            modified_toml = true;
                            output_dir = fix_path("/tmp/");
                        }
                    }
                }
                let using_temp = output_dir == "/tmp/";
                let fallback_option = table["fallback"].as_str();
                match fallback_option {
                    Some(x) => {
                        if x == "" {
                            if fix {
                                fallback = prompt_fallback(using_temp);
                                modified_toml = true;
                            } else {
                                println!("Failed to read fallback wallpaper, ignoring...");
                                fallback = String::from("");
                            }
                        } else {
                            fallback = String::from(x);
                        }
                    },
                    _none => {
                        if fix {
                            fallback = prompt_fallback(using_temp);
                        } else {
                            println!("Failed to read fallback wallpaper, ignoring...");
                            fallback = String::from("");
                            modified_toml = true;
                        }
                    }
                }
                /*
                fallback = match fallback_option {
                    Some(x) => String::from(x),
                    _none => String::from("")
                };
                */
                
                for i in links_step_2 {
                    links.push(i);
                    web_url.push(String::from(""));
                    local_url.push(String::from(fallback.as_str()));
                }
                if modified_toml {
                    let _ = create_settings_file(
                        &settings_path, 
                        links.join(",").as_str(), 
                        output_dir.as_str(), 
                        fallback.as_str()
                    );
                }
            },
            Err(_) => {
                println!("Failed to read walltaker.toml");
                if fix {
                    println!("Creating new file at {}", settings_path);
                    let links_in = prompt_links();
                    for i in links_in {
                        links.push(i);
                        web_url.push(String::from(""));
                    }

                    output_dir = prompt_output_dir();
                    let using_temp = output_dir == "/tmp/";

                    fallback = prompt_fallback(using_temp);
                    for i in &links {
                        if i != "" {
                            local_url.push(String::from(fallback.as_str()));
                        }
                    }

                    let _ = create_settings_file(
                        &settings_path, 
                        links.join(",").as_str(), 
                        output_dir.as_str(), 
                        fallback.as_str()
                    );
                } else {
                    println!("Run this file with the arguement --fix to create a new file.");
                    return
                }
                return
            }
        }
        
    } else {
        println!("walltaker.toml doesn't exist, creating...");
        println!("You can edit {} later to modify these settings.", settings_path);

        let links_in = prompt_links();
        for i in links_in {
            links.push(i);
            web_url.push(String::from(""));
        }
        
        output_dir = prompt_output_dir();
        let using_temp = output_dir == "/tmp/";

        
        fallback = prompt_fallback(using_temp);
        for i in &links {
            if i != "" {
                local_url.push(String::from(fallback.as_str()));
            }
        }

        let _ = create_settings_file(
            &settings_path, 
            links.join(",").as_str(), 
            output_dir.as_str(), 
            fallback.as_str()
        );
        println!("I'd recommend adding this file as a startup script.");
        thread::sleep(Duration::from_secs(5));
    }
    let sleep_time = Duration::from_secs(20);
    let mut i = 0;
    let mut first_loop = true;
    let using_temp = output_dir == "/tmp/";
    if using_temp {
        //On KDE, setting a wallpaper to what it's currently set as does nothing.
        //If the wallpaper fails to load, it uses the fallback.

        //These 2 mean if a file is stored in /tmp/, it's deleted, the fallback is used,
        //the wallpaper is redownloaded, and is never changed back to the correct wallpaper.

        //Manually setting it to a different wallpaper and changing it back fixes this.
        update_wallpapers(&local_url, fallback.as_str());
    }
    loop {
        i += 1;
        if i >= links.len() {
            i = 0;
            if first_loop {
                first_loop = false;
            }
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
                Ok(_) => {
                    if !first_loop || links.len() - 1 == i {
                        update_wallpapers(&local_url, fallback.as_str());
                    }
                },
                Err(e) => println!("{:?}", e)
            }
        }
        if !first_loop {
            thread::sleep(sleep_time);
        }
    }
}