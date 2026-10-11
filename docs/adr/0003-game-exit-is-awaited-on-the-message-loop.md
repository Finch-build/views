# The Engine waits for a game to exit on its own message loop

Gaming mode ends when the game's process exits. The plan proposed `RegisterWaitForSingleObject`, which parks a thread-pool thread on the process handle, and asked for that cost to be measured.

The Windows host instead passes the handle to `MsgWaitForMultipleObjectsEx`, the call its message loop already blocks in. The Engine keeps its one thread, there is nothing to measure, and the wait is still a kernel wait with no polling.
