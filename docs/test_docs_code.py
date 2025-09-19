#!/usr/bin/env python3
"""
Extract and run Python code blocks from Markdown documentation files.
"""

import re
import sys
import traceback
from pathlib import Path

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

def test_markdown_file(file_path):
    """Test all Python code blocks in a markdown file."""
    print(f"Testing Python code blocks in: {file_path}")
    print("=" * 60)
    
    # Read the markdown file
    with open(file_path, 'r', encoding='utf-8') as f:
        content = f.read()
    
    # Extract Python code blocks
    code_blocks = extract_python_code_blocks(content)
    
    if not code_blocks:
        print("No Python code blocks found.")
        return True
    
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
        return True
    else:
        print("❌ Some code blocks failed")
        return False

if __name__ == "__main__":
    if len(sys.argv) != 2:
        print("Usage: python test_docs_code.py <markdown_file>")
        sys.exit(1)
    
    markdown_file = Path(sys.argv[1])
    if not markdown_file.exists():
        print(f"Error: File {markdown_file} does not exist")
        sys.exit(1)
    
    success = test_markdown_file(markdown_file)
    sys.exit(0 if success else 1)
