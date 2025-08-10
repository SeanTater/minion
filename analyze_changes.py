#!/usr/bin/env python3
"""
Git change analysis script for pathfinding feature commit organization.
Categorizes unstaged changes to help create clean, focused commits.
"""

import subprocess
import re
from collections import defaultdict
from pathlib import Path

def run_git_command(cmd):
    """Run a git command and return output lines."""
    result = subprocess.run(cmd, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        print(f"Error running: {cmd}")
        print(result.stderr)
        return []
    return result.stdout.strip().split('\n') if result.stdout.strip() else []

def analyze_file_diff(filepath):
    """Analyze the diff for a file to categorize the type of changes."""
    diff_lines = run_git_command(f"git diff '{filepath}'")
    
    if not diff_lines:
        return "no_changes"
    
    # Count different types of changes
    formatting_changes = 0
    content_changes = 0
    
    for line in diff_lines:
        if line.startswith('@@'):
            continue
        elif line.startswith('+') or line.startswith('-'):
            if line.startswith('+++') or line.startswith('---'):
                continue
                
            # Remove leading +/- for analysis
            content = line[1:]
            
            # Check for formatting-only changes
            if (
                # Trailing whitespace changes
                re.match(r'^\s*$', content) or
                # Pure whitespace differences
                content.strip() == '' or
                # Import reordering (common formatting)
                (('use ' in content or 'mod ' in content) and 
                 any(keyword in content for keyword in ['pub mod', 'use ', 'mod '])) or
                # Comment formatting
                content.strip().startswith('//') or
                # Brace-only lines
                content.strip() in ['{', '}', '};', '},'] or
                # Pure whitespace/indent changes
                len(content) - len(content.lstrip()) != len(content.rstrip()) - len(content.strip())
            ):
                formatting_changes += 1
            else:
                content_changes += 1
    
    # Categorize based on change ratio
    total_changes = formatting_changes + content_changes
    if total_changes == 0:
        return "no_changes"
    elif formatting_changes > content_changes * 3:  # 75%+ formatting
        return "formatting_only"
    elif content_changes > 0:
        return "content_changes"
    else:
        return "formatting_only"

def categorize_file(filepath):
    """Categorize a file based on its path and changes."""
    path = Path(filepath)
    
    # Check for pathfinding-related files
    pathfinding_patterns = [
        r'pathfinding',
        r'navigation',
        r'obstacle',
        r'collision.*trait',
        r'environment.*integration'
    ]
    
    if any(re.search(pattern, str(path), re.IGNORECASE) for pattern in pathfinding_patterns):
        return "pathfinding"
    
    # Check for generated/build files
    if str(path) in ['uv.lock', 'Cargo.lock', '.pre-commit-config.yaml']:
        return "generated"
    
    # Check for documentation files
    if path.suffix == '.md' and 'notes/' not in str(path):
        change_type = analyze_file_diff(filepath)
        if change_type == "formatting_only":
            return "formatting"
        else:
            return "docs"
    
    # Check for config files
    if path.name in ['config.toml', '.pre-commit-config.yaml']:
        return "config"
    
    # Check the actual diff content for other files
    change_type = analyze_file_diff(filepath)
    
    if change_type == "formatting_only":
        return "formatting"
    elif 'src/' in str(path):
        # For source files, check if they contain pathfinding-related changes
        diff_content = ' '.join(run_git_command(f"git diff '{filepath}'"))
        if any(keyword in diff_content.lower() for keyword in 
               ['pathfind', 'obstacle', 'navigation', 'collision_trait']):
            return "pathfinding"
        else:
            return "unrelated_code"
    else:
        return "other"

def main():
    print("🔍 Analyzing unstaged changes for pathfinding commit organization\n")
    
    # Get all modified files (unstaged)
    modified_files = run_git_command("git diff --name-only")
    untracked_files = run_git_command("git ls-files --others --exclude-standard")
    
    # Combine all files to analyze
    all_files = modified_files + untracked_files
    
    # Categorize files
    categories = defaultdict(list)
    
    for filepath in all_files:
        if not filepath.strip():
            continue
        category = categorize_file(filepath)
        categories[category].append(filepath)
    
    # Report results
    print("📊 CHANGE CATEGORIZATION RESULTS")
    print("=" * 50)
    
    for category, files in sorted(categories.items()):
        emoji_map = {
            "pathfinding": "🎯",
            "formatting": "✨", 
            "generated": "🔧",
            "docs": "📝",
            "config": "⚙️",
            "unrelated_code": "🔀",
            "other": "❓"
        }
        
        emoji = emoji_map.get(category, "📁")
        print(f"\n{emoji} {category.upper().replace('_', ' ')} ({len(files)} files):")
        for f in sorted(files):
            print(f"  • {f}")
    
    # Generate recommendations
    print("\n\n🎯 RECOMMENDATIONS")
    print("=" * 50)
    
    if categories.get("pathfinding"):
        print("\n✅ INCLUDE IN PATHFINDING COMMIT:")
        for f in sorted(categories["pathfinding"]):
            print(f"  git add '{f}'")
    
    if categories.get("formatting"):
        print("\n🧹 HANDLE FORMATTING SEPARATELY:")
        print("  # Option 1: Reset formatting changes")
        for f in sorted(categories["formatting"]):
            print(f"  git checkout -- '{f}'")
        print("\n  # Option 2: Stash for later formatting commit")
        print("  git stash push -m 'formatting changes' -- " + " ".join(f"'{f}'" for f in categories["formatting"]))
    
    if categories.get("generated"):
        print("\n🚫 ADD TO .gitignore:")
        gitignore_entries = []
        for f in categories["generated"]:
            if f == "uv.lock":
                gitignore_entries.append("uv.lock")
            elif f == ".pre-commit-config.yaml":
                print(f"  # Note: {f} might be intentional - review manually")
        
        if gitignore_entries:
            print("  # Add these to .gitignore:")
            for entry in gitignore_entries:
                print(f"  echo '{entry}' >> .gitignore")
    
    if categories.get("unrelated_code") or categories.get("other"):
        print("\n⚠️  REVIEW MANUALLY:")
        for category in ["unrelated_code", "other"]:
            if categories.get(category):
                print(f"  # {category.replace('_', ' ').title()}:")
                for f in sorted(categories[category]):
                    print(f"  git diff '{f}'  # Review this file")
    
    print("\n\n🚀 SUGGESTED WORKFLOW:")
    print("=" * 50)
    print("1. Stage pathfinding-related files (see above)")
    print("2. Reset or stash formatting-only changes") 
    print("3. Review unrelated changes for separate commits")
    print("4. Update .gitignore for generated files")
    print("5. Commit clean pathfinding feature")

if __name__ == "__main__":
    main()