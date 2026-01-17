# cc-timeline

**cc-timeline** is a tool for tracing, analyzing, and visualizing the execution timeline of command-line programs (such as build commands). It supports converting trace data to Chrome Trace format for browser-based visualization, and provides advanced features like thread reduction and dependency analysis.

## Features

- Trace the execution time and arguments of any command
- Batch conversion and visualization of trace files
- Remap trace events to a specified number of threads for easier concurrency analysis
- Generate dependency flow events to enhance visualization
- Output compatible with Chrome Trace Viewer (JSON format)

## Installation

Requires Rust 1.59+.

```sh
git clone https://github.com/yourname/cc-timeline.git
cd cc-timeline
cargo build --release
```

The executable will be at `target/release/cc-timeline`.

## Usage

### 1. Trace command execution

```sh
cc-timeline trace -- gcc -c foo.c -o foo.o
```

This runs `gcc -c foo.c -o foo.o` and saves the trace to `trace.ndjson` (use `-o` to specify the output file).

### 2. Convert to Chrome Trace format

```sh
cc-timeline convert -i trace.ndjson -o chrome_trace.json
```

Converts the trace file to Chrome Trace format for visualization in [chrome://tracing](chrome://tracing) or [perfetto](https://ui.perfetto.dev/) (**recommended**).

### 3. Thread reduction (thread compaction)

```sh
cc-timeline convert -i trace.ndjson -o chrome_trace.json --compact 8
```

Remaps all trace events to 8 threads for easier concurrency analysis.

If you use `--compact` without a parameter, the tool will automatically use the hardware thread count.

### 4. Generate dependency flow events

```sh
cc-timeline convert -i trace.ndjson -o chrome_trace.json --flow
```

Generates dependency flow events to enhance visualization of dependencies.

## Integration with Existing Projects

You can easily integrate **cc-timeline** into your existing build systems to trace compilation commands and visualize the build process.

### With Make

For Make-based projects, simply override compiler variables (such as `CC`, `CXX`, etc.) to use `cc-timeline trace`. For example:

```sh
make CC="cc-timeline trace -- gcc" CXX="cc-timeline trace -- g++"
```

This will wrap each compilation command with `cc-timeline trace`, allowing you to collect trace data for every build step.

### With CMake

For CMake projects, you need to create a wrapper script and set it as the launcher for your compiler.

1. **Create a wrapper script (e.g., `cc_timeline_gcc.sh`):**

    ```sh
    #!/bin/bash
    # get current script directory
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    exec cc-timeline trace -o "$SCRIPT_DIR/trace.ndjson" -- "$@"
    ```

    Make the script executable:

    ```sh
    chmod +x cc_timeline_gcc.sh
    ```

2. **Set the compiler launcher in your CMake configuration:**

    ```sh
    cmake -DCMAKE_C_COMPILER_LAUNCHER=./cc_timeline_gcc.sh .
    ```

This will ensure all compiler invocations are traced by **cc-timeline** during the build.

---

After building, you can use the collected trace files with the `convert` command to generate Chrome Trace format for visualization.

## Command Line Arguments

### Main commands

- `trace`: Trace a command and record its execution timeline
- `convert`: Convert a trace file to Chrome Trace format

### Common options

| Option               | Description                                                        |
|----------------------|--------------------------------------------------------------------|
| `-o, --output`       | Output file path (trace file for `trace`, Chrome Trace for `convert`) |
| `-i, --input`        | Input trace file path (used with `convert`)                        |
| `--flow`             | Generate dependency flow events                                    |
| `--compact [N]`      | Compact threads to N; if omitted, uses hardware thread count       |

## Visualization

1. Open [Perfetto UI](https://ui.perfetto.dev/) in your browser.
2. Load the `chrome_trace.json` file to view the timeline.

## Contributing

Issues and PRs are welcome!

## License

MIT