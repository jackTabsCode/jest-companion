# jest-companion

Run [jest-lua](https://github.com/jsdotlua/jest-lua) tests from the command line. My successor to [testez-companion-cli](https://github.com/jackTabsCode/testez-companion-cli)!

## Installation

There are several ways you can install jest-companion and its plugin, but here's how I'd do it with Mise:

Run:

```bash
mise use github:jacktabscode/jest-companion
mise use github:jacktabscode/drillbit # This installs the plugin for you
```

In `drillbit.toml`:

```toml
[plugins.jest-companion]
# This won't auto-update
github = "https://github.com/jackTabsCode/jest-companion/releases/download/v0.2.1/plugin.rbxm"
```

Then, you can run `drillbit` to install the plugin, and `jest-companion` to run your tests in Studio.

## Usage

This tool spins up a server that tells the Studio plugin to run tests, and sends back the results.

Run `jest-companion --help` to see the available options. Several of [jest-lua's runCLI options](https://jsdotlua.github.io/jest-lua/cli) can be set through the CLI, like `--testNamePattern` (which is why I made this tool!)

## Notes

- The plugin does not forward logs to the CLI. See the Studio output for these.
- jest-companion takes in the table of test results that jest-lua gives it, formats it nicely and prints it in your console. This means that the output may be different than jest-lua's Studio output, sometimes in a less-than-desirable way. If the output you receive from the CLI seems weird or incorrect, file an issue.
