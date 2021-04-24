# Minecraft Server CLI ✨

This is a CLI tool used to edit and persist your Minecraft server command-line settings and start your Minecraft server with these settings.

## Installation

In order to install the `minecraft-server-cli` tool, you need to download one of the binaries or build it yourself using the `cargo build` command and then locate the binary at `./target/debug/minecraft-server-cli`

## Usage

### Linux

```bash
# First let's give it execute permission.
chmod +x minecraft-server-cli

# Next let's move the binary to a location that is in your $PATH.
mv minecraft-server-cli ~/.local/bin/

# Now you should be able to run it. Let's get some help info.
minecraft-server-cli --help

# To use this tool, provide it the name of your server's .jar file.
minecraft-server-cli minecraft-server.jar

# If you've installed your Minecraft server somewhere other than "~/.minecraft/server/",
# you can set the directory with the second command-line option.
minecraft-server-cli minecraft-server.jar /opt/path/to/minecraft/server/

# If you want, you can alias out this command to make it shorter.
# Just add your alias to .bashrc, .zshrc, or your shell's equivalent file.
alias msc="minecraft-server-cli minecraft-server.jar"
```

### Windows

First, I would suggest making a `bin\` folder in your Minecraft server directory. Then, move the `minecraft-server-cli.exe` binary into it. Now you need to add that folder path to the `PATH` environment variable.

1. Copy the abosolute path to your `bin\` folder.
1. Press the Windows key and type in "path".
1. Select the "Edit the system environment variables" option. This will open the System Properties window.
1. Near the bottom of this window, click "Environment Variables". This will open the Environment Variables window.
1. In that window, locate the variable labelled "Path" and double-click it.
1. Click the "New" button on the right-hand side.
1. Paste or type the absolute path to your `bin\` folder.
1. Hit "OK" in all of the windows.

```powershell
# Usage is exactly the same as above but with Windows directories and defaults.
minecraft-server-cli --help

# This will default to "%AppData%\.minecraft\server\" for the server directory.
minecraft-server-cli minecraft-server.jar

# But you can still specify your own directory path — relative or absolute.
minecraft-server-cli minecraft-server.jar .\server\

# If you want to create an alias for Powershell as well, you can open your code editor:
code $((Split-Path $profile -Parent) + "\profile.ps1")
# or with Notepad
notepad $((Split-Path $profile -Parent) + "\profile.ps1")

# Then add a function to the file with your command:
Function msc {minecraft-server-cli minecraft-server.jar}
```

## Uninstallation

To uninstall this program, please reverse all of the above changes you may have made.

## License

License MIT © [Chris Brown](https://github.com/ChrisBrownie55)
