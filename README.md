# FileMonitor

## SOW

Create a command-line program that accepts an optional argument: “-d path”. If the path is not supplied, it defaults to “~/inbox/”. If the path does not exist, it should be created. We refer to this path as “INBOX” in the rest of the document.

Program workflow:
1. Scan the folder recursively and print to stdout all the files found and their last modification date in the following format: “[Date Time] PATH”, where PATH is a relative path to INBOX.
2. Start monitoring INBOX for file changes. When an event occurs, print it to stdout in the following format: “[EVENT] PATH”, where EVENT is one of the following [NEW, MOD, DEL].
3. Continue monitoring until the user inputs Ctrl-C.
4. Once Ctrl-C is detected, print to stdout the contents of INBOX again in the same format, without rescanning or any other FS operations.

Bonus points for:
1. Using tokio
2. Using structured error handling
3. Not using mutexes
4. Having separation of concerns
