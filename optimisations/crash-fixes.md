### **1. Prevent Stack Overflow**
The stack overflow error suggests that the application is recursively processing too many files or folders, overwhelming the stack. To fix this:
- **Use Iterative Algorithms**: Replace recursive file traversal with iterative methods (e.g., using a stack or queue data structure) to avoid stack overflow.
- **Limit Recursion Depth**: If recursion is necessary, impose a maximum depth limit to prevent excessive stack usage.

### **2. Add Safety Checks for File Selection**
Before performing any selection action (e.g., `*` or `space`), you should:
- **Count the Number of Files/Folders**: Traverse the directory structure and count the number of items that would be selected.
- **Impose a Reasonable Limit**: If the count exceeds a predefined threshold (e.g., 100 files), abort the action and notify the user.

#### **b. Check Selection Limit**
Before performing the selection, call the `count_files_in_directory` function and compare the result to your limit (e.g., 100 files).

#### **c. Notify the User**
If the limit is exceeded, display a modal or message to the user:

### **3. User Feedback**
When the selection limit is exceeded, provide clear and actionable feedback to the user. For example:
- **Modal Dialog**: Display a modal with the message:  
  `"You attempted to select XXX files. The maximum allowed is 100. Aborted."`
- **Log Message**: Log the event for debugging purposes.

### **4. Optimize Performance**
If counting files in large directories is slow, consider:
- **Caching File Counts**: Cache the number of files in frequently accessed directories.
- **Asynchronous Counting**: Perform the count in a background thread to avoid blocking the UI.