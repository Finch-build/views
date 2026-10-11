// The games list holds executable file names, which core compares without regard to case.

const fold = (exe: string) => exe.trim().toLowerCase();

export const isListed = (games: readonly string[], exe: string) => games.some((game) => fold(game) === fold(exe));

/** Whether `exe` can join `games`: it is not blank and not already there. */
export const canAdd = (games: readonly string[], exe: string) => exe.trim() !== "" && !isListed(games, exe);
