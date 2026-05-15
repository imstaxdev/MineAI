import { invoke } from "@tauri-apps/api/core";

export type Profile = {
  id: number;
  username: string;
  offlineUuid: string;
  minecraftVersion: string;
  ramMb: number;
  javaPath?: string | null;
  createdAt: string;
  updatedAt: string;
};

export type JavaInfo = {
  path: string;
  version?: string | null;
  source: string;
};

export type ProfileSettings = {
  minecraftVersion?: string;
  ramMb?: number;
  javaPath?: string | null;
};

export type LaunchResult = {
  pid: number;
  logFile: string;
  commandPreview: string[];
};

export type InstallVersionReport = {
  version: string;
  downloadedFiles: number;
  reusedFiles: number;
  versionDir: string;
  libraries: number;
  assets: number;
  natives: number;
};

export type MinecraftVersionItem = {
  id: string;
  versionType: string;
  latest: boolean;
  installed: boolean;
};

export type ImportedFile = {
  fileName: string;
  destination: string;
  hashes: {
    blake3: string;
    sha256: string;
  };
  reusedExisting: boolean;
};

export type ModpackImportReport = {
  name: string;
  gameVersion?: string | null;
  loader?: string | null;
  filesDeclared: number;
  downloadsRequired: number;
  overridesExtracted: number;
};

export type RepairReport = {
  profileId: number;
  ensuredDirectories: string[];
};

export const api = {
  createProfile: (username: string) => invoke<Profile>("create_profile", { username }),
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  updateProfileSettings: (profileId: number, settings: ProfileSettings) =>
    invoke<Profile>("update_profile_settings", { profileId, settings }),
  detectJava: () => invoke<JavaInfo>("detect_java"),
  isVersionInstalled: (version: string) => invoke<boolean>("is_version_installed", { version }),
  listMinecraftVersions: () => invoke<MinecraftVersionItem[]>("list_minecraft_versions"),
  installVersion: (version: string) => invoke<InstallVersionReport>("install_version", { version }),
  prepareVersionForLaunch: (version: string) =>
    invoke<InstallVersionReport>("prepare_version_for_launch", { version }),
  launchProfile: (profileId: number) => invoke<LaunchResult>("launch_profile", { profileId }),
  importMod: (profileId: number, file: string) => invoke<ImportedFile>("import_mod", { profileId, file }),
  importShader: (profileId: number, file: string) => invoke<ImportedFile>("import_shader", { profileId, file }),
  importModpack: (profileId: number, file: string) =>
    invoke<ModpackImportReport>("import_modpack", { profileId, file }),
  repairProfile: (profileId: number) => invoke<RepairReport>("repair_profile", { profileId }),
};
