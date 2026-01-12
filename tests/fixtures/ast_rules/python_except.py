# Test fixture for no-bare-except rule
# This file contains bare except: clauses at specific lines for testing

def example1():
    try:
        risky_operation()
    except:  # Line 7 - should be detected
        pass

def example2():
    try:
        another_operation()
    except:  # Line 13 - should be detected
        print("error occurred")

def nested_example():
    try:
        outer_operation()
    except:  # Line 20 - should be detected
        try:
            recovery()
        except:  # Line 23 - should be detected
            pass

# This should not trigger - specific exception
def clean_code():
    try:
        operation()
    except ValueError:
        print("value error")
    except Exception as e:
        print(f"error: {e}")
