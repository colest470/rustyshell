use crate::exec::find_in_path;

pub fn builtins (args: &[&str]) {
    let target = args.first().copied().unwrap_or("");

    if matches!(target, "type" | "exit" | "echo") {
        println!("{} is a shell builtin", target);
    } else if let Some(path) = find_in_path(target) {
        println!("{} is {}", target, path.display());
    } else {
        println!("{}: not found", target);
    }
}

// pub fn echo(args: &[&str]) {
//     // initial implementation
//     println!("{}", args.join(" "));

//     let final_echo: str = String::new("");
//     for element in &args {
//         if element.starts_with("'") && element.ends_with("'") {
//             final_echo += " " + &element[1..&element.len() - 1];
//         } 

//         for (i, character) in &final_echo {
//             if character == "'" && &final_echo[i+1] == "'"{
//                 final_echo =
//             }
//         }
//     } 

//     println!(final_echo);
// }