# The Settings UI updates both executables

Releases are frequent, and running an installer by hand for each one is not acceptable. Something has to fetch the new version.

The Settings UI does it, with Tauri's updater: when it opens it asks GitHub for the newest release, and on the user's say-so downloads the installer, checks its signature and runs it. The installer already stops the Engine and replaces both executables. The Engine gains no network code, so its size and its zero idle wakeups are untouched, at the price that a machine where the Settings UI is never opened is never updated.

A release is cut by the version alone: the Release workflow publishes whenever main holds a version with no release, so the version in the two Cargo workspaces is the only thing to change and no tag is pushed by hand.

An installed copy trusts one signing key. Its private half lives in the `TAURI_SIGNING_PRIVATE_KEY` secret; if it is lost, no installed copy can be updated again and every user must reinstall by hand.
