use anyhow::Result;

use crate::cache::cache::Cache;
use crate::cache::cache::CacheEntry;
use crate::cache::generators::CacheGenerator;

/// Shell cache generator
pub struct ShellCacheGenerator;

impl CacheGenerator for ShellCacheGenerator {
    fn name(&self) -> &'static str {
        "shell"
    }

    fn file_extension(&self) -> &'static str {
        "sh"
    }

    fn generate_content(&self, cache: &Cache) -> Result<String> {
        let mut content = String::new();
        content.push_str("#!/bin/bash\n");
        content.push_str("# Auto-generated by dotgk sync\n");
        content.push_str("# Do not edit manually\n");
        content.push_str("# Source this file to get gatekeeper functions\n\n");

        // Sort entries by name for consistent output
        let mut entries: Vec<(&String, &CacheEntry)> = cache.cache.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));

        // Create an associative array to store gatekeeper values
        content.push_str("# Gatekeeper values stored in associative array\n");
        content.push_str("declare -A _DOTGK_VALUES=(\n");
        for (name, entry) in entries {
            let value = if entry.value { "true" } else { "false" };
            content.push_str(&format!("  [\"{}\"]=\"{}\"\n", name, value));
        }
        content.push_str(")\n\n");

        // Add a helper function to check gatekeeper values
        content.push_str("# Helper function to check gatekeeper values\n");
        content.push_str("dotgk_check() {\n");
        content.push_str("  local name=\"$1\"\n");
        content.push_str("  local value=\"${_DOTGK_VALUES[$name]:-false}\"\n");
        content.push_str("  [[ \"$value\" == \"true\" ]]\n");
        content.push_str("}\n");

        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::generators::test_utils::create_test_cache;

    #[test]
    fn test_shell_generator() -> Result<()> {
        let generator = ShellCacheGenerator;
        let cache = create_test_cache();

        let content = generator.generate_content(&cache)?;

        assert!(content.contains("#!/bin/bash"));
        assert!(content.contains("# Auto-generated by dotgk sync"));
        assert!(content.contains("declare -A _DOTGK_VALUES=("));
        assert!(content.contains("[\"another_gk\"]=\"false\""));
        assert!(content.contains("[\"test-gk\"]=\"true\""));
        assert!(content.contains("dotgk_check() {"));

        Ok(())
    }
}
