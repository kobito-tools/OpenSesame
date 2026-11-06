# OpenSesame!

English | [日本語](README.ja.md)

<img src="kobito/icon-1024.png" width="96" align="right" alt="">

OpenSesame! is a launcher for macOS that appears only while you hold down a key.

Hold the activation keys (Left Control + Left Option by default) and a keyboard-shaped panel appears. Press one of your assigned keys to open an app, a folder or a website, run a command, or snap the frontmost window to the left half, right half and more.

OpenSesame! is part of [kobito-tools](https://github.com/kobito-tools), a series of small Mac apps.

![Demo](docs/images/demo.gif)

## What it can do

| Type | What happens when you press the key |
|---|---|
| Window layout | Moves and resizes the frontmost window: halves, quarters, thirds, two thirds, center, maximize |
| App | Launches the app, or brings it to the front. It can also hide the app if it is already in front |
| Folder | Opens the folder in Finder |
| Website | Opens the URL in your default browser |
| Command | Runs a one-line terminal command, such as `uv run app.py` |

Each key can hold two items: one on the main layer and one on the Shift layer.

## Requirements

- macOS 11 or later (Apple Silicon or Intel)

## Download and install

1. Download `OpenSesame!_x.y.z_universal.dmg` from the [latest release](../../releases/latest). `x.y.z` is the version number.
2. Open the `.dmg` and drag OpenSesame! into the Applications folder.
3. Launch OpenSesame! from the Applications folder.

> **Note:** Launch it from the Applications folder. If you run it from somewhere else, such as Downloads, macOS may not keep the permissions below.

You can also build the app yourself from the source code. A self-built app does not show the warning below. See [Build from source](#build-from-source).

### First launch: the “unidentified developer” warning

OpenSesame! is not notarized by Apple, so the first launch shows a warning such as “cannot verify the developer”. Close the warning, open **System Settings → Privacy & Security**, and click **Open Anyway** near the bottom. After that, it opens normally.

If macOS says the app “is damaged and can’t be opened”, run this in Terminal and launch it again:

```bash
xattr -dr com.apple.quarantine "/Applications/OpenSesame!.app"
```

### Allow Accessibility

OpenSesame! needs Accessibility permission to notice your key presses while you are using other apps. On first launch, the settings window shows a guide.

1. Click **Open Accessibility settings** in the guide.
2. Click **+** below the list and add OpenSesame! from the Applications folder.
3. Turn on the switch next to OpenSesame!.

Within a few seconds the guide disappears and OpenSesame! starts listening. You do not need to restart it.

### Allow Automation

The first time you use a window layout, macOS asks whether “OpenSesame!” may control “System Events”. Click **Allow**.

### Switch the settings window to English

The settings window starts in Japanese. Use the language menu at the top right to choose **English**. The menu bar items follow the language the next time OpenSesame! starts.

## How to use

### Basic keys

| Keys | What happens |
|---|---|
| Hold the activation keys | Shows the panel |
| Activation keys + an assigned key | Runs that item: opens an app, folder or website, runs a command, or lays out the window |
| Activation keys + Shift | Switches to the Shift layer. Release Shift to go back |
| Activation keys + Shift + an assigned key | Runs the item on the Shift layer |
| Activation keys + `.` (period) | Opens the settings window |
| Release the activation keys | Hides the panel |

Closing the settings window does not quit OpenSesame!. It keeps running in the background.

### Assign a key

1. In the keyboard at the top of the settings window, click a free key.
2. Choose **Window layout**, **Launch an app**, **Open a folder**, **Open a website** or **Run a command**.
3. For an app or a folder, pick it in the dialog. For a website, enter the URL (`https://` can be left out) and a display name. For a command, see [Run a command](#run-a-command).

You can also use **＋ App**, **＋ Folder**, **＋ Website** and **＋ Command** at the top right. They assign a free key for you.

In the list at the bottom of the settings window, you can change an item’s display name and key. Drag an assigned key onto another key on the keyboard to move it. Click **Icon** to use any PNG image as the icon.

For apps, turn on **Hide if in front** to hide the app when you press its key while it is already in front. Each press then switches between showing and hiding it.

### Run a command

You can assign the same command you would type in Terminal, for example `uv run app.py` to start a Python app.

| Field | What to enter |
|---|---|
| Command (one line) | The command, as you would type it in Terminal. No line breaks. Join several commands with `&&` to run them in order |
| Working folder (optional) | The folder to run the command in. Use **Choose…** to pick one. Empty means your home folder |
| Display name | The name shown on the panel and in the list. Empty means the first word of the command |

- The command runs through a login shell (`/bin/zsh -l`). The `PATH` you set in `~/.zprofile` is loaded, so commands such as `uv` that work in Terminal are found. If you set `PATH` only in `~/.zshrc`, the command may not be found. Add it to `~/.zprofile` too, or write the full path (for example `~/.local/bin/uv run app.py`).
- OpenSesame! starts the command and does not wait for it to finish. If the command cannot start, or exits with an error right away (within 3 seconds), the settings window opens and shows the exit code and the last line of the error at the bottom.
- The command’s output is not shown.
- Use **Edit** in the list to change a command. Use **Icon** to pick a PNG image (▶️ is shown otherwise).

> **Warning:** A registered command runs as is every time you press its key. Register only commands you understand.

### Use the Shift layer

Switch the keyboard at the top right to **Shift layer** to assign a second set of items. The same key can hold one item on the main layer and another on the Shift layer. To run a Shift-layer item, hold Shift as well as the activation keys.

If Shift is one of your activation keys, that Shift does not switch layers. For example, if Left Shift is an activation key, use Right Shift for the Shift layer.

### Change the activation keys

1. Hover over the **Activation keys** field in the settings window and press delete.
2. Press the modifier keys you want together, then release them.

You can combine up to three of Control, Option, Command and Shift. Left and right keys are told apart. Shift alone cannot be used. To cancel, press Esc before step 2.

### Change the language and keyboard layout

At the top right of the settings window, switch the language (日本語 / English) and the keyboard layout (JIS / ANSI).

### Open at login

At the top right of the settings window, set **Open at login** to **On**. OpenSesame! then starts in the menu bar when you log in, without opening the settings window. Set it to **Off** to stop. If you added OpenSesame! to **System Settings → General → Login Items** yourself before, remove it there so it is not started twice.

### Quit

Click the OpenSesame! icon in the menu bar and choose **Quit OpenSesame!**.

## Troubleshooting

### The panel does not appear

- Make sure Accessibility permission is turned on. If it stopped working after an update, select OpenSesame! in the Accessibility list, remove it with **−**, and add it again.
- If you use VoiceOver, the default activation keys (Control + Option) are the same as the VoiceOver modifier. Change the activation keys.

### Windows are not laid out

Open **System Settings → Privacy & Security → Automation** and make sure “System Events” is turned on under OpenSesame!.

### “Target not found” appears and settings cannot be saved

A registered app or folder was moved or deleted, or it is on an external drive that is not connected. Remove the item from the list, or connect the drive.

### Nothing happens when I press a command’s key

If the bottom of the settings window says the command stopped right away with exit code 127, the command was not found. Check your `PATH` or use the full path, as described in [Run a command](#run-a-command). The last line of the command’s error follows the exit code.

### “Some items are turned off” appears

The config file (`config.json`) holds entries that the settings window would not save. For example, a website URL that does not start with `http://` or `https://`, or an app or folder location written as a URL. Something other than OpenSesame! may have changed the config file.

The items listed there do nothing when you press their keys. If you do not recognize an item, remove it from the list. If you do, remove it and register it again (a command can be fixed with **Edit**). You can remove or fix them one at a time and save.

### My settings went back to the defaults

If the config file was damaged, OpenSesame! keeps the original as `config.json.broken-<number>` and starts with the default settings. You can find that file in the settings folder listed in [Uninstall](#uninstall).

## Uninstall

1. Quit OpenSesame!.
2. Move OpenSesame! from the Applications folder to the Trash. If you installed it with `setup.sh`, it is inside the `kobito-tools` folder.
3. In **System Settings → Privacy & Security → Accessibility**, select OpenSesame! and remove it with **−**.

To remove your settings as well, delete this folder:

- `~/Library/Application Support/io.github.kobito-tools.opensesame`

To also remove the launch info file (see [Working with other apps](#working-with-other-apps)), delete this file:

- `~/Library/Application Support/kobito-tools/apps/opensesame.json`

## Privacy

OpenSesame! does not use the network. Key presses are used only to detect the activation keys and the assigned keys. They are never recorded or sent anywhere. Your settings are stored only on your Mac.

## For developers

### Build from source

To install without the warning on first launch, clone this repository and run `./setup.sh` in Terminal (or double-click `Setup.command` in Finder). An app you build yourself has no quarantine flag from a download, so the warning does not appear.

- You need Xcode Command Line Tools, Node.js 20 or later, and Rust.
  - If Rust is missing, it asks whether to install it with the official installer (rustup).
  - If Xcode Command Line Tools or Node.js is missing, it shows how to install them and stops.
- After the build, it asks whether to move `OpenSesame!.app` into the `kobito-tools` folder in Applications (`/Applications/kobito-tools/`). If the folder does not exist, it creates it and gives it the series icon. If `OpenSesame!.app` is already there, it asks before replacing it. You can say no and still use the app: `OpenSesame!.app` stays in the repository folder, and you can move it yourself.
- An older `OpenSesame!.app` directly in `/Applications` is left alone. If you no longer use it, move it to the Trash yourself.
- At the end, it writes the launch info using the installed `OpenSesame!.app` (see [Working with other apps](#working-with-other-apps)). It does not open a window or stop a running OpenSesame!. It also records the repository location as a symbolic link at `kobito-tools/repos/opensesame` under `~/Library/Application Support/`.
- If another OpenSesame! repository (with the same app ID) is already recorded there, it stops before building and changes nothing. Decide which one to use and follow the instructions it shows.
- `./setup.sh --help` shows the options, and `./setup.sh --dry-run` shows what it would do without changing anything. It is safe to run more than once.
- After a rebuild, remove OpenSesame! from the Accessibility list with **−** and add it again. Each build has a new signature, so the old permission no longer applies.

For build details and the source layout, see [docs/DEVELOPMENT.md](docs/DEVELOPMENT.md) (in Japanese).

### Working with other apps

OpenSesame! connects to other apps only through the data formats defined in `docs/kobito-spec/CONTRACTS.md`.

Each time it starts, OpenSesame! writes how to launch it and how to show it (name, color and icon) as “launch info” on your Mac. Other apps in the series read this file to show OpenSesame! as a launch button. OpenSesame! is a utility, so it does not show up inside other apps.

- Location: `~/Library/Application Support/kobito-tools/apps/opensesame.json`
- To write the file without opening a window, run `OpenSesame!.app/Contents/MacOS/OpenSesame --write-launch-info`. It writes the file and exits.
- Only how to launch and show the app is written. Your settings and registered items are not.
- To print the same content to standard output without writing the file, use `--metadata`.
- `kobito-app.json` at the repository root marks this as a kobito-tools app. The icons (`icon.svg`, `icon-1024.png`) are in `kobito/`. It is shipped as an app, so its kind is `"native"` (see `docs/kobito-spec/APP_PACKAGE.md`).

### Issues

Bug reports and requests are welcome in [Issues](../../issues).

### How this was made

OpenSesame! was developed with the help of ChatGPT (OpenAI) and Claude (Anthropic). This is a personal project and is not affiliated with OpenAI or Anthropic.

### License

OpenSesame! is released under the [MIT License](LICENSE).

The software is provided “as is”, without warranty of any kind. The author is not liable for any damage arising from its use. See [LICENSE](LICENSE) for details.
