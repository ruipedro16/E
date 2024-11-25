use std::collections::{HashMap, VecDeque};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn collect_c_files(dir: &Path) -> Result<HashMap<PathBuf, String>, Box<dyn std::error::Error>> {
    let mut files_map = HashMap::new();

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let sub_files = collect_c_files(&path)?;
            files_map.extend(&mut sub_files.into_iter());
        } else {
            if let Some(ext) = path.extension() {
                if ext == "c" {
                    let content = fs::read_to_string(&path)?;
                    files_map.insert(path, content);
                }
            }
        }
    }

    Ok(files_map)
}

fn extract_includes(file_content: &str) -> Vec<String> {
    // returns the files included by a given file
    let mut includes = Vec::new();
    for line in file_content.lines() {
        if line.starts_with("#include") {
            if let Some(start) = line.find('"') {
                if let Some(end) = line[start + 1..].find('"') {
                    includes.push(line[start + 1..start + 1 + end].to_string());
                }
            }
        }
    }
    includes
}

fn build_dependency_graph(files: &HashMap<PathBuf, String>) -> HashMap<PathBuf, Vec<PathBuf>> {
    let mut graph = HashMap::new();
    for (file, content) in files {
        let dependencies = extract_includes(content);
        let mut resolved_deps = Vec::new();
        for dep in dependencies {
            let dep_path = file.parent().unwrap().join(dep);
            if dep_path.exists() {
                resolved_deps.push(dep_path);
            }
        }
        graph.insert(file.clone(), resolved_deps);
    }
    graph
}

fn topological_sort(graph: &HashMap<PathBuf, Vec<PathBuf>>) -> Result<Vec<PathBuf>, String> {
    let mut in_degree = HashMap::new();
    let mut adj_list = HashMap::new();

    for (node, edges) in graph {
        in_degree.entry(node.clone()).or_insert(0);
        for edge in edges {
            *in_degree.entry(edge.clone()).or_insert(0) += 1;
            adj_list
                .entry(node.clone())
                .or_insert_with(Vec::new)
                .push(edge.clone());
        }
    }

    let mut queue = VecDeque::new();
    for (node, &degree) in &in_degree {
        if degree == 0 {
            queue.push_back(node.clone());
        }
    }

    let mut sorted = Vec::new();
    while let Some(node) = queue.pop_front() {
        sorted.push(node.clone());
        if let Some(edges) = adj_list.get(&node) {
            for edge in edges {
                if let Some(degree) = in_degree.get_mut(edge) {
                    *degree -= 1;
                    if *degree == 0 {
                        queue.push_back(edge.clone());
                    }
                }
            }
        }
    }

    if sorted.len() == graph.len() {
        Ok(sorted)
    } else {
        Err("Cycle detected in graph".to_string())
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} <path>", args[0]);
        std::process::exit(1);
    }

    let dir = Path::new(&args[1]);

    if !dir.exists() {
        eprintln!("Error: Path '{}' does not exist.", args[1]);
        std::process::exit(1);
    }

    if !dir.is_dir() {
        eprintln!("Error: Path '{}' is not a directory.", args[1]);
        std::process::exit(1);
    }

    let files_map = collect_c_files(dir).expect("Failed to collect C files"); // Key: path to file, Value: content of file

    let dependency_graph = build_dependency_graph(&files_map);

    match topological_sort(&dependency_graph) {
        Ok(sorted) => {
            for file in sorted {
                println!("{}", file.display());
            }
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
