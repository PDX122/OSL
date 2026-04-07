#!/bin/bash
# OSL Compiler Test Script

OSLC="./target/release/oslc"

echo "=========================================="
echo "OSL Compiler Test Suite"
echo "=========================================="
echo ""

# Test 1: Simple file
echo "Test 1: Simple file"
$OSLC check test-project/simple.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 2: Minimal file
echo ""
echo "Test 2: Minimal file"
$OSLC check test-project/minimal.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 3: Basic file
echo ""
echo "Test 3: Basic file"
$OSLC check test-project/basic2.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 4: Core features test
echo ""
echo "Test 4: Core features test"
$OSLC check test-project/test_core.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 5: Type check
echo ""
echo "Test 5: Type check"
$OSLC typecheck test-project/test_core.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 6: Build
echo ""
echo "Test 6: Build"
$OSLC build test-project/test_core.osl -o test-project/output.rs
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 7: Return statement
echo ""
echo "Test 7: Return statement"
$OSLC check test-project/return_test.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 8: Variable then function
echo ""
echo "Test 8: Variable then function"
$OSLC check test-project/t2.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 9: List syntax
echo ""
echo "Test 9: List syntax"
$OSLC check test-project/list_test.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

# Test 10: Working features
echo ""
echo "Test 10: Working features"
$OSLC check test-project/test_working.osl
if [ $? -eq 0 ]; then
    echo "✓ PASSED"
else
    echo "✗ FAILED"
fi

echo ""
echo "=========================================="
echo "Tests completed"
echo "=========================================="