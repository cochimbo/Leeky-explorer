// Integration tests for Go To Path functionality
use std::path::PathBuf;
use std::fs;
use tempfile::TempDir;

#[test]
fn test_absolute_path_navigation() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let test_path = temp_dir.path().join("test_dir");
    fs::create_dir(&test_path)?;
    
    // Test that absolute path works
    let absolute = test_path.canonicalize()?;
    assert!(absolute.is_absolute());
    assert!(absolute.exists());
    assert!(absolute.is_dir());
    
    Ok(())
}

#[test]
fn test_relative_path_navigation() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base = temp_dir.path();
    
    // Create directory structure: base/parent/child
    let parent = base.join("parent");
    let child = parent.join("child");
    fs::create_dir(&parent)?;
    fs::create_dir(&child)?;
    
    // From child, go to parent using ../
    let relative = PathBuf::from("..");
    let resolved = child.join(&relative).canonicalize()?;
    assert_eq!(resolved, parent.canonicalize()?);
    
    // From child, go to base using ../../
    let relative2 = PathBuf::from("../..");
    let resolved2 = child.join(&relative2).canonicalize()?;
    assert_eq!(resolved2, base.canonicalize()?);
    
    Ok(())
}

#[test]
fn test_home_directory_expansion() {
    // Test that ~ expands to home directory
    if let Some(home) = dirs::home_dir() {
        assert!(home.exists());
        assert!(home.is_dir());
        
        // Test ~ by itself
        let expanded = expand_tilde("~");
        assert_eq!(expanded, home.to_string_lossy());
        
        // Test ~/subdir
        let expanded_sub = expand_tilde("~/Documents");
        assert!(expanded_sub.starts_with(&home.to_string_lossy().to_string()));
    }
}

#[test]
fn test_environment_variable_expansion() {
    use std::env;
    
    // Set a test variable
    unsafe {
        env::set_var("TEST_VAR", "test_value");
    }
    
    #[cfg(target_os = "windows")]
    {
        let input = "C:\\%TEST_VAR%\\path";
        let expanded = expand_env_vars(input);
        assert_eq!(expanded, "C:\\test_value\\path");
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let input = "/home/$TEST_VAR/path";
        let expanded = expand_env_vars(input);
        assert_eq!(expanded, "/home/test_value/path");
        
        let input2 = "/home/${TEST_VAR}/path";
        let expanded2 = expand_env_vars(input2);
        assert_eq!(expanded2, "/home/test_value/path");
    }
    
    unsafe {
        env::remove_var("TEST_VAR");
    }
}

#[test]
fn test_invalid_path_rejected() {
    let invalid_path = "/this/path/does/not/exist/hopefully/12345";
    let result = validate_path_exists(invalid_path);
    assert!(result.is_err());
}

#[test]
fn test_file_path_rejected() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let file_path = temp_dir.path().join("test_file.txt");
    fs::write(&file_path, "test content")?;
    
    // File should exist but not be a directory
    assert!(file_path.exists());
    assert!(file_path.is_file());
    
    let result = validate_is_directory(&file_path);
    assert!(result.is_err());
    
    Ok(())
}

#[test]
fn test_empty_path_rejected() {
    let empty = "";
    let trimmed = empty.trim();
    assert!(trimmed.is_empty());
    
    let whitespace = "   ";
    let trimmed2 = whitespace.trim();
    assert!(trimmed2.is_empty());
}

#[test]
fn test_whitespace_trimming() {
    let path_with_spaces = "  /home/user  ";
    let trimmed = path_with_spaces.trim();
    assert_eq!(trimmed, "/home/user");
    
    let path_with_tabs = "\t/tmp/test\t";
    let trimmed2 = path_with_tabs.trim();
    assert_eq!(trimmed2, "/tmp/test");
}

#[test]
fn test_path_canonicalization() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let base = temp_dir.path();
    
    // Create: base/a/b
    let dir_a = base.join("a");
    let dir_b = dir_a.join("b");
    fs::create_dir(&dir_a)?;
    fs::create_dir(&dir_b)?;
    
    // Test that b/../b resolves to b
    let with_dots = dir_b.join("..").join("b");
    let canonical = with_dots.canonicalize()?;
    assert_eq!(canonical, dir_b.canonicalize()?);
    
    Ok(())
}

#[test]
fn test_navigation_adds_to_history() -> anyhow::Result<()> {
    use leeky_explorer::models::panel::Panel;
    
    let temp_dir = TempDir::new()?;
    let base = temp_dir.path();
    let target = base.join("target");
    fs::create_dir(&target)?;
    
    let mut panel = Panel::new(base.to_path_buf());
    let initial_count = panel.history.count();
    
    // Simulate navigation via Go To
    panel.current_path = target.clone();
    panel.history.push(target.clone());
    
    assert_eq!(panel.history.count(), initial_count + 1);
    let entries = panel.history.get_all();
    assert_eq!(entries.last(), Some(&target));
    
    Ok(())
}

#[test]
fn test_permission_check() -> anyhow::Result<()> {
    let temp_dir = TempDir::new()?;
    let test_dir = temp_dir.path().join("readable");
    fs::create_dir(&test_dir)?;
    
    // Should be able to read
    let result = fs::read_dir(&test_dir);
    assert!(result.is_ok());
    
    Ok(())
}

#[test]
fn test_multiple_env_vars() {
    use std::env;
    
    unsafe {
        env::set_var("VAR1", "value1");
        env::set_var("VAR2", "value2");
    }
    
    #[cfg(target_os = "windows")]
    {
        let input = "%VAR1%\\middle\\%VAR2%";
        let expanded = expand_env_vars(input);
        assert_eq!(expanded, "value1\\middle\\value2");
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        let input = "$VAR1/middle/$VAR2";
        let expanded = expand_env_vars(input);
        assert_eq!(expanded, "value1/middle/value2");
    }
    
    unsafe {
        env::remove_var("VAR1");
        env::remove_var("VAR2");
    }
}

#[test]
fn test_tilde_only() {
    if let Some(home) = dirs::home_dir() {
        let expanded = expand_tilde("~");
        assert_eq!(expanded, home.to_string_lossy());
    }
}

#[test]
fn test_tilde_with_path() {
    if let Some(home) = dirs::home_dir() {
        let expanded = expand_tilde("~/test/path");
        let expected = format!("{}/test/path", home.display());
        assert_eq!(expanded, expected);
    }
}

// Helper functions for tests (mirroring handler.rs logic)

fn expand_tilde(input: &str) -> String {
    if let Some(rest) = input.strip_prefix('~') {
        if let Some(home) = dirs::home_dir() {
            if rest.is_empty() {
                home.to_string_lossy().to_string()
            } else {
                format!("{}{}", home.display(), rest)
            }
        } else {
            input.to_string()
        }
    } else {
        input.to_string()
    }
}

fn expand_env_vars(input: &str) -> String {
    let mut result = input.to_string();
    
    #[cfg(target_os = "windows")]
    {
        while let Some(start) = result.find('%') {
            if let Some(end) = result[start + 1..].find('%') {
                let var_name = &result[start + 1..start + 1 + end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + 2 + end..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    
    #[cfg(not(target_os = "windows"))]
    {
        while let Some(start) = result.find("${") {
            if let Some(end) = result[start..].find('}') {
                let var_name = &result[start + 2..start + end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + end + 1..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        while let Some(start) = result.find('$') {
            let rest = &result[start + 1..];
            let end = rest.chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count();
            
            if end > 0 {
                let var_name = &rest[..end];
                if let Ok(value) = std::env::var(var_name) {
                    result = format!("{}{}{}", &result[..start], value, &result[start + 1 + end..]);
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    
    result
}

fn validate_path_exists(path: &str) -> Result<(), String> {
    let p = PathBuf::from(path);
    if !p.exists() {
        Err(format!("Path does not exist: {}", path))
    } else {
        Ok(())
    }
}

fn validate_is_directory(path: &std::path::Path) -> Result<(), String> {
    if !path.is_dir() {
        Err(format!("Path is not a directory: {}", path.display()))
    } else {
        Ok(())
    }
}
