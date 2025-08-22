use anyhow::Result;
use colored::*;
use crate::config::{Config, Provider};

pub struct ProviderManager;

impl ProviderManager {
    pub fn list_providers(config: &Config) -> Result<()> {
        if config.providers.is_empty() {
            println!("{}", "No service providers configured".yellow());
            return Ok(());
        }

        println!("{}", "Configured service providers:".blue().bold());
        println!();

        for (name, provider) in &config.providers {
            let is_current = config.current_provider
                .as_ref() == Some(name);

            let marker = if is_current { "●".green() } else { "○".white() };
            let name_color = if is_current { name.green().bold() } else { name.white() };
            
            println!("  {} {}", marker, name_color);
            println!("    API URL: {}", provider.api_url.cyan());
            println!("    Token: {}****", &provider.token[..provider.token.len().min(8)].dimmed());
            
            if is_current {
                println!("    {}", "(currently active)".green().italic());
            }
            println!();
        }

        Ok(())
    }

    pub fn add_provider(config: &mut Config, name: String, api_url: String, token: String) -> Result<()> {
        if config.providers.contains_key(&name) {
            println!("{} Service provider '{}' already exists, overwriting", "⚠️".yellow(), name.yellow());
        }

        config.add_provider(name.clone(), api_url, token);
        config.save()?;

        println!("{} Successfully added service provider '{}'", "✅".green(), name.green().bold());
        Ok(())
    }

    pub fn remove_provider(config: &mut Config, name: &str) -> Result<()> {
        if !config.providers.contains_key(name) {
            println!("{} Service provider '{}' does not exist", "❌".red(), name.red());
            return Ok(());
        }

        config.remove_provider(name);
        config.save()?;

        println!("{} Successfully removed service provider '{}'", "🗑️".green(), name.green().bold());
        Ok(())
    }

    pub fn use_provider(config: &mut Config, name: &str) -> Result<()> {
        if !config.providers.contains_key(name) {
            println!("{} Service provider '{}' does not exist", "❌".red(), name.red());
            return Ok(());
        }

        if let Some(current) = &config.current_provider {
            if current == name {
                println!("{} Already using service provider '{}'", "ℹ️".blue(), name.blue().bold());
                return Ok(());
            }
        }

        let provider = config.providers.get(name).unwrap().clone();
        
        // Set environment variables
        config.set_current_provider(name);
        config.save()?;

        println!("{} Switched to service provider '{}'", "🔄".green(), name.green().bold());
        println!("  API URL: {}", provider.api_url.cyan());
        println!();
        println!("{} To take effect in current terminal, run:", "💡".blue().bold());
        
        Self::set_environment_variables(&provider)?;
        
        Ok(())
    }

    fn set_environment_variables(provider: &Provider) -> Result<()> {
        // Immediately set environment variables for current process
        std::env::set_var("ANTHROPIC_AUTH_TOKEN", &provider.token);
        std::env::set_var("ANTHROPIC_BASE_URL", &provider.api_url);
        
        // Output environment variable commands that can be executed by shell
        println!("export ANTHROPIC_AUTH_TOKEN=\"{}\"", provider.token);
        println!("export ANTHROPIC_BASE_URL=\"{}\"", provider.api_url);
        
        Ok(())
    }

    pub fn use_provider_eval(config: &mut Config, name: &str) -> Result<()> {
        if !config.providers.contains_key(name) {
            eprintln!("# Error: Service provider '{}' does not exist", name);
            return Ok(());
        }

        let provider = config.providers.get(name).unwrap().clone();
        
        config.set_current_provider(name);
        config.save()?;

        // Only output environment variable commands
        println!("export ANTHROPIC_AUTH_TOKEN=\"{}\"", provider.token);
        println!("export ANTHROPIC_BASE_URL=\"{}\"", provider.api_url);
        
        Ok(())
    }

    
    pub fn check_environment(config: &Config) -> Result<()> {
        println!("{}", "🔍 Checking environment variable status".blue().bold());
        println!();
        
        // Check current environment variables
        let current_api_key = std::env::var("ANTHROPIC_AUTH_TOKEN");
        let current_api_url = std::env::var("ANTHROPIC_BASE_URL");
        
        println!("{}", "Current environment variables:".cyan().bold());
        match &current_api_key {
            Ok(key) => {
                let masked_key = if key.len() > 8 {
                    format!("{}****", &key[..8])
                } else {
                    "****".to_string()
                };
                println!("  ANTHROPIC_AUTH_TOKEN: {}", masked_key.green());
            }
            Err(_) => {
                println!("  ANTHROPIC_AUTH_TOKEN: {}", "Not set".red());
            }
        }
        
        match &current_api_url {
            Ok(url) => {
                println!("  ANTHROPIC_BASE_URL: {}", url.green());
            }
            Err(_) => {
                println!("  ANTHROPIC_BASE_URL: {}", "Not set".red());
            }
        }
        
        println!();
        
        // Check configuration status
        if let Some(current_provider) = &config.current_provider {
            if let Some(provider) = config.providers.get(current_provider) {
                println!("{}", "CCE configuration status:".cyan().bold());
                println!("  Current provider: {}", current_provider.green().bold());
                println!("  Configured URL: {}", provider.api_url.cyan());
                
                // Verify if environment variables match configuration
                let env_matches = match (&current_api_key, &current_api_url) {
                    (Ok(env_key), Ok(env_url)) => {
                        env_key == &provider.token && env_url == &provider.api_url
                    }
                    _ => false,
                };
                
                if env_matches {
                    println!("  Status: {}", "✅ Environment variables match configuration".green());
                } else {
                    println!("  Status: {}", "⚠️ Environment variables do not match configuration".yellow());
                    println!("  Suggestion: Run 'cce use {}' to reset", current_provider.cyan());
                }
            } else {
                println!("{}", "❌ Configuration error: Current provider does not exist".red());
            }
        } else {
            println!("{}", "CCE configuration status:".cyan().bold());
            println!("  Current provider: {}", "None selected".yellow());
            if !config.providers.is_empty() {
                println!("  Suggestion: Use 'cce use <provider-name>' to select a provider");
            } else {
                println!("  Suggestion: Use 'cce add' to add a service provider");
            }
        }
        
        Ok(())
    }
    
    pub fn output_shellenv() -> Result<()> {
        // Get current executable path
        let current_exe = std::env::current_exe()
            .unwrap_or_else(|_| std::path::PathBuf::from("cce"));
        let cce_path = current_exe.display();
        
        // Output complete shell function definition
        println!(r#"cce() {{
    local cce_binary="{}"
    
    if [[ "$1" == "use" && -n "$2" ]]; then
        local env_output=$("$cce_binary" use "$2" --eval 2>/dev/null)
        if [[ $? -eq 0 && -n "$env_output" ]]; then
            eval "$env_output"
            echo "⚡ Switched to service provider '$2'"
            echo "✅ Environment variables are now active in current terminal"
        else
            "$cce_binary" "$@"
        fi
    else
        "$cce_binary" "$@"
    fi
}}"#, cce_path);
        
        Ok(())
    }
}