// Test fixture for no-any rule
// This file contains 'any' type annotations at specific lines for testing

function example1(param: any): void {  // Line 4 - should be detected
    console.log(param);
}

function example2(): any {  // Line 8 - should be detected
    return 42;
}

let variable: any = "hello";  // Line 12 - should be detected

interface Example {
    field: any;  // Line 16 - should be detected
}

type Callback = (arg: any) => void;  // Line 19 - should be detected

class MyClass {
    private data: any;  // Line 22 - should be detected

    constructor(value: any) {  // Line 24 - should be detected
        this.data = value;
    }
}

// This should not trigger - no any
function cleanCode(param: string): number {
    return param.length;
}
