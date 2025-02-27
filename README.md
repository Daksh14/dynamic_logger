# Install rust

Rustup: https://rustup.rs/

# Script modification

I had to modify the script to pipe commands to my CLI, the change is simple

```diff
# Generate burst of log entries
    for ((i=0; i<burst_size; i++)); do
+        generate_log_entry
    done
```

# Run 

```
./generator.sh | cargo r
```

This should output the following

```
Log Analysis Report (Last Updated: 2025-02-27 17:49:59.231322 -05:00)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Runtime Stats:
Entries Processed: 83742
Current Rate: 285 entires/sec (Peak: 557 entires/sec)
Adaptive window: 120 sec

Pattern Analysis:

Error: 33.290344152277235% (27967 entires)
Debug: 33.39662296099926% (27878 entires)
Info: 33.311838742805286% (27896 entires)

Self Evolving alerts
High error rate:

```
