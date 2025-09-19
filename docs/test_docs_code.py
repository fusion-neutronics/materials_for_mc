#!/usr/bin/env python3
"""
Extract and run Python code blocks from Markdown documentation files.
Run with
    python docs/test_docs_code.py docs/source/python_usage.md
"""

import re
import sys
import traceback
from pathlib import Path
import pytest

def extract_python_code_blocks(markdown_content):
    """Extract Python code blocks from markdown content."""
    # Pattern to match Python code blocks
    pattern = r'```python\n(.*?)\n```'
    matches = re.findall(pattern, markdown_content, re.DOTALL)
    return matches

def run_code_blocks(code_blocks, context=None):
    """Run code blocks in sequence with shared context."""
    if context is None:
        context = {}
    
    results = []
    
    for i, code in enumerate(code_blocks):
        print(f"\n--- Running code block {i+1} ---")
        print(code)
        print("-" * 40)
        
        try:
            # Execute the code in the shared context
            exec(code, context)
            print("✓ Success")
            results.append(True)
        except Exception as e:
            print(f"✗ Error: {e}")
            traceback.print_exc()
            results.append(False)
    
    return results

def test_markdown_file():
    """Test all Python code blocks in the python_usage.md file."""
    # Get the path to the documentation file
    docs_dir = Path(__file__).parent
    file_path = docs_dir / "source" / "python_usage.md"
    
    if not file_path.exists():
        pytest.skip(f"Documentation file not found: {file_path}")
    
    print(f"Testing Python code blocks in: {file_path}")
    print("=" * 60)
    
    # Read the markdown file
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract Python code blocks
    code_blocks = extract_python_code_blocks(content)
    
    if not code_blocks:
        print("No Python code blocks found.")
        return
    
    print(f"Found {len(code_blocks)} Python code blocks.")
    
    # Run the code blocks
    results = run_code_blocks(code_blocks)
    
    # Summary
    print("\n" + "=" * 60)
    print("Summary:")
    successful = sum(results)
    total = len(results)
    print(f"✓ {successful}/{total} code blocks ran successfully")
    
    if successful == total:
        print("🎉 All code blocks passed!")
    else:
        print("❌ Some code blocks failed")
        # Fail the test if any code blocks failed
        assert False, f"Only {successful}/{total} code blocks passed"

if __name__ == "__main__":
    if len(sys.argv) == 2:
        # Command line usage
        markdown_file = Path(sys.argv[1])
        if not markdown_file.exists():
            print(f"Error: File {markdown_file} does not exist")
            sys.exit(1)
        
        # Read the markdown file
        with open(markdown_file, 'r', encoding='utf-8') as f:
            content = f.read()
        
        # Extract Python code blocks
        code_blocks = extract_python_code_blocks(content)
        
        if not code_blocks:
            print("No Python code blocks found.")
            sys.exit(0)
        
        print(f"Found {len(code_blocks)} Python code blocks.")
        
        # Run the code blocks
        results = run_code_blocks(code_blocks)
        
        # Summary
        print("\n" + "=" * 60)
        print("Summary:")
        successful = sum(results)
        total = len(results)
        print(f"✓ {successful}/{total} code blocks ran successfully")
        
        if successful == total:
            print("🎉 All code blocks passed!")
            sys.exit(0)
        else:
            print("❌ Some code blocks failed")
            sys.exit(1)
    else:
        print("Usage: python test_docs_code.py <markdown_file>")
        print("Or run with pytest to test the default documentation file.")
        sys.exit(1)
