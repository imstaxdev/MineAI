import { For, Show, createMemo, createSignal, onMount } from "solid-js";
import { InstallVersionReport, LaunchResult, ModpackImportReport, Profile, api } from "./api";
import logo from "./assets/mineia.png";

const versionCards = [
  { id: "26.1.2", tag: "Ultima release", note: "Actual" },
  { id: "26.1.1", tag: "Release", note: "Reciente" },
  { id: "26.1", tag: "Release", note: "Base 26" },
  { id: "1.21.11", tag: "Release", note: "Nueva" },
  { id: "1.21.10", tag: "Release", note: "Estable" },
  { id: "1.21.9", tag: "Release", note: "Estable" },
  { id: "1.21.8", tag: "Release", note: "Instalada" },
  { id: "1.21.7", tag: "Release", note: "Extra" },
];

const navItems = [
  { id: "play", label: "Inicio", mark: "P" },
  { id: "versions", label: "Versiones", mark: "V" },
  { id: "mods", label: "Mods", mark: "M" },
] as const;

type Panel = (typeof navItems)[number]["id"];

export default function App() {
  const [profiles, setProfiles] = createSignal<Profile[]>([]);
  const [selectedId, setSelectedId] = createSignal<number | null>(null);
  const [username, setUsername] = createSignal("");
  const [version, setVersion] = createSignal(versionCards[0].id);
  const [importPath, setImportPath] = createSignal("");
  const [installReport, setInstallReport] = createSignal<InstallVersionReport | null>(null);
  const [launch, setLaunch] = createSignal<LaunchResult | null>(null);
  const [modpack, setModpack] = createSignal<ModpackImportReport | null>(null);
  const [busy, setBusy] = createSignal(false);
  const [message, setMessage] = createSignal("Listo para jugar");
  const [panel, setPanel] = createSignal<Panel>("play");

  const selectedProfile = createMemo(() => {
    const id = selectedId();
    return profiles().find((profile) => profile.id === id) ?? profiles()[0] ?? null;
  });

  const selectedVersionCard = createMemo(
    () => versionCards.find((item) => item.id === version()) ?? versionCards[0],
  );

  const localVersionCount = createMemo(() => versionCards.length);

  onMount(() => {
    void refreshProfiles();
  });

  async function runTask(task: () => Promise<void>) {
    setBusy(true);
    try {
      await task();
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    } finally {
      setBusy(false);
    }
  }

  async function refreshProfiles() {
    await runTask(async () => {
      const data = await api.listProfiles();
      setProfiles(data);
      if (!selectedId() && data.length > 0) {
        selectProfile(data[0]);
      }
      setMessage(data.length > 0 ? "Perfiles cargados" : "Crea tu perfil");
    });
  }

  function selectProfile(profile: Profile) {
    setSelectedId(profile.id);
    setVersion(profile.minecraftVersion);
  }

  function chooseVersion(id: string) {
    setVersion(id);
    setMessage(`Version seleccionada: ${id}`);
  }

  async function createProfile() {
    await runTask(async () => {
      const profile = await api.createProfile(username());
      setProfiles([profile, ...profiles()]);
      setUsername("");
      selectProfile(profile);
      setMessage(`Perfil listo: ${profile.username}`);
    });
  }

  async function persistSelectedVersion(profile: Profile) {
    if (profile.minecraftVersion === version()) {
      return profile;
    }

    const updated = await api.updateProfileSettings(profile.id, {
      minecraftVersion: version(),
    });
    setProfiles(profiles().map((item) => (item.id === updated.id ? updated : item)));
    selectProfile(updated);
    return updated;
  }

  async function launchSelected() {
    const profile = selectedProfile();
    if (!profile) return;

    await runTask(async () => {
      const updated = await persistSelectedVersion(profile);
      const result = await api.launchProfile(updated.id);
      setLaunch(result);
      setMessage("Minecraft se esta abriendo");
    });
  }

  async function installSelectedVersion() {
    await runTask(async () => {
      const result = await api.installVersion(version());
      setInstallReport(result);
      setMessage(`Version lista: ${result.version}`);
    });
  }

  async function installAllVersions() {
    await runTask(async () => {
      let last: InstallVersionReport | null = null;
      for (const item of versionCards) {
        setMessage(`Preparando ${item.id}`);
        last = await api.installVersion(item.id);
      }
      if (last) {
        setInstallReport(last);
      }
      setMessage("Versiones listas en local");
    });
  }

  async function importFile(kind: "mod" | "shader" | "modpack") {
    const profile = selectedProfile();
    const file = importPath().trim();
    if (!profile || !file) return;

    await runTask(async () => {
      if (kind === "mod") {
        const result = await api.importMod(profile.id, file);
        setMessage(result.reusedExisting ? "Mod listo" : `Mod agregado: ${result.fileName}`);
      }
      if (kind === "shader") {
        const result = await api.importShader(profile.id, file);
        setMessage(result.reusedExisting ? "Shader listo" : `Shader agregado: ${result.fileName}`);
      }
      if (kind === "modpack") {
        const result = await api.importModpack(profile.id, file);
        setModpack(result);
        setMessage(`Modpack agregado: ${result.name}`);
      }
    });
  }

  return (
    <main class="min-h-screen overflow-hidden bg-panel text-ink">
      <div class="grid min-h-screen grid-cols-[76px_288px_1fr] max-xl:grid-cols-[72px_1fr]">
        <nav class="flex flex-col items-center border-r border-line/80 bg-black/30 py-5">
          <img src={logo} alt="MineIA" class="mb-8 h-11 w-11 rounded-md object-cover shadow-soft" />
          <div class="flex flex-1 flex-col gap-3">
            <For each={navItems}>
              {(item) => (
                <button
                  class={`flex h-12 w-12 items-center justify-center rounded-md border text-sm font-black transition ${
                    panel() === item.id
                      ? "border-blue2 bg-blue text-white shadow-soft"
                      : "border-transparent bg-white/5 text-muted hover:border-blue hover:text-ink"
                  }`}
                  title={item.label}
                  onClick={() => setPanel(item.id)}
                >
                  {item.mark}
                </button>
              )}
            </For>
          </div>
          <button
            class="h-12 w-12 rounded-md border border-transparent bg-white/5 text-sm font-black text-muted transition hover:border-blue2 hover:text-ink disabled:opacity-50"
            disabled={busy()}
            title="Recargar"
            onClick={refreshProfiles}
          >
            R
          </button>
        </nav>

        <aside class="border-r border-line bg-surface px-5 py-5 max-xl:hidden">
          <div class="mb-7">
            <p class="text-xs font-bold uppercase tracking-[0.24em] text-blue2">MineIA</p>
            <h1 class="mt-1 text-3xl font-black">Launcher</h1>
          </div>

          <div class="mb-4 rounded-md border border-line bg-panel/70 p-3">
            <div class="mb-3">
              <span class="rounded-md border border-blue/50 bg-blue/15 px-3 py-2 text-center text-xs font-black uppercase text-blue2">
                Modo offline
              </span>
            </div>
            <div class="flex gap-2">
              <input
                class="min-w-0 flex-1 rounded-md border border-line bg-black/20 px-3 py-3 text-sm text-ink outline-none placeholder:text-muted focus:border-blue2"
                placeholder="Usuario"
                value={username()}
                onInput={(event) => setUsername(event.currentTarget.value)}
              />
              <button
                class="rounded-md bg-blue px-4 text-sm font-black text-white transition hover:bg-blue2 hover:text-panel disabled:opacity-50"
                disabled={busy() || username().trim().length < 3}
                onClick={createProfile}
              >
                +
              </button>
            </div>
          </div>

          <div class="mb-3 flex items-center justify-between">
            <span class="text-xs font-bold uppercase tracking-[0.16em] text-muted">Perfiles</span>
            <span class="text-xs font-bold text-blue2">{profiles().length}</span>
          </div>

          <div class="space-y-2">
            <For each={profiles()}>
              {(profile) => (
                <button
                  class={`w-full rounded-md border px-3 py-3 text-left transition ${
                    selectedProfile()?.id === profile.id
                      ? "border-blue2 bg-blue/20 shadow-soft"
                      : "border-line bg-surface2 hover:border-blue"
                  }`}
                  onClick={() => selectProfile(profile)}
                >
                  <div class="flex items-center justify-between gap-3">
                    <span class="font-bold">{profile.username}</span>
                    <span class="rounded-sm border border-blue/40 bg-blue/10 px-2 py-1 text-[11px] font-bold text-blue2">
                      {profile.minecraftVersion}
                    </span>
                  </div>
                  <p class="mt-2 text-xs text-muted">Offline local</p>
                </button>
              )}
            </For>
          </div>
        </aside>

        <section class="min-w-0 overflow-y-auto px-7 py-6 max-sm:px-4">
          <div class="mb-5 flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-bold text-blue2">{message()}</p>
              <h2 class="truncate text-3xl font-black max-sm:text-2xl">
                <Show when={selectedProfile()} fallback="Crea un perfil">
                  {(profile) => `Bienvenido, ${profile().username}`}
                </Show>
              </h2>
            </div>
            <div class="hidden items-center gap-3 rounded-md border border-line bg-surface px-4 py-3 xl:flex">
              <span class="text-sm text-muted">Locales</span>
              <strong>{localVersionCount()}</strong>
            </div>
          </div>

          <Show when={panel() === "play"}>
            <section class="relative mb-5 min-h-[330px] overflow-hidden rounded-md border border-line bg-hero shadow-soft">
              <div class="absolute inset-0 bg-[radial-gradient(circle_at_25%_25%,rgba(56,189,248,0.18),transparent_34%),radial-gradient(circle_at_78%_18%,rgba(37,99,235,0.22),transparent_34%)]" />
              <img
                src={logo}
                alt=""
                class="absolute right-10 top-1/2 h-72 w-72 -translate-y-1/2 rounded-md object-cover opacity-20 max-lg:right-0 max-lg:h-56 max-lg:w-56"
              />
              <div class="relative z-10 flex min-h-[330px] flex-col justify-between p-8 max-sm:p-5">
                <div class="max-w-2xl">
                  <p class="mb-3 text-xs font-black uppercase tracking-[0.28em] text-blue2">
                    {selectedVersionCard().tag}
                  </p>
                  <h3 class="text-6xl font-black leading-none max-lg:text-5xl max-sm:text-4xl">MineIA</h3>
                  <p class="mt-4 max-w-xl text-base font-semibold text-muted">
                    Launcher offline y open source. Perfiles locales, versiones verificadas y ajustes livianos.
                  </p>
                  <div class="mt-5 flex flex-wrap gap-2">
                    <span class="rounded-sm border border-grass/40 bg-grass/10 px-3 py-2 text-xs font-black uppercase text-grass">
                      Offline
                    </span>
                    <span class="rounded-sm border border-blue2/40 bg-blue2/10 px-3 py-2 text-xs font-black uppercase text-blue2">
                      Bajo consumo
                    </span>
                    <span class="rounded-sm border border-ember/40 bg-ember/10 px-3 py-2 text-xs font-black uppercase text-ember">
                      Local
                    </span>
                  </div>
                </div>

                <div class="flex flex-wrap items-end justify-between gap-4">
                  <div class="grid grid-cols-2 gap-3 max-sm:w-full">
                    <div class="rounded-md border border-line bg-black/25 px-4 py-3">
                      <p class="text-xs font-bold uppercase text-muted">Version</p>
                      <p class="mt-1 text-2xl font-black">{selectedVersionCard().id}</p>
                    </div>
                    <div class="rounded-md border border-line bg-black/25 px-4 py-3">
                      <p class="text-xs font-bold uppercase text-muted">Modo</p>
                      <p class="mt-1 text-2xl font-black text-grass">Offline</p>
                    </div>
                  </div>

                  <div class="flex gap-3 max-sm:w-full max-sm:flex-col">
                    <button
                      class="h-14 rounded-md border border-blue bg-blue/15 px-6 text-sm font-black text-blue2 transition hover:bg-blue hover:text-white disabled:opacity-50"
                      disabled={busy()}
                      onClick={installSelectedVersion}
                    >
                      Preparar
                    </button>
                    <button
                      class="h-16 min-w-48 rounded-md bg-blue px-10 text-lg font-black text-white shadow-soft transition hover:bg-blue2 hover:text-panel disabled:opacity-50 max-sm:w-full"
                      disabled={busy() || !selectedProfile()}
                      onClick={launchSelected}
                    >
                      Jugar
                    </button>
                  </div>
                </div>
              </div>
            </section>

            <div class="grid grid-cols-[minmax(0,1fr)_340px] gap-5 max-2xl:grid-cols-1">
              <section class="rounded-md border border-line bg-surface p-5 shadow-soft">
                <div class="mb-4 flex items-center justify-between gap-3">
                  <div>
                    <h3 class="text-lg font-black">Versiones locales</h3>
                  <p class="text-sm text-muted">Selecciona una tarjeta, prepara la version y toca Jugar.</p>
                  </div>
                  <button
                    class="rounded-md border border-line bg-surface2 px-4 py-3 text-sm font-bold text-ink transition hover:border-blue2 disabled:opacity-50"
                    disabled={busy()}
                    onClick={installAllVersions}
                  >
                    Preparar todas
                  </button>
                </div>
                <VersionGrid version={version()} onChoose={chooseVersion} />
              </section>

              <StatusPanel
                selectedVersion={selectedVersionCard()}
                installReport={installReport()}
                launch={launch()}
                modpack={modpack()}
              />
            </div>
          </Show>

          <Show when={panel() === "versions"}>
            <section class="rounded-md border border-line bg-surface p-6 shadow-soft">
              <div class="mb-5 flex flex-wrap items-center justify-between gap-3">
                <div>
                  <h3 class="text-2xl font-black">Versiones</h3>
                  <p class="text-sm text-muted">Todo queda cacheado en tu PC y se reutiliza por hash.</p>
                </div>
                <button
                  class="rounded-md bg-blue px-5 py-3 text-sm font-black text-white transition hover:bg-blue2 hover:text-panel disabled:opacity-50"
                  disabled={busy()}
                  onClick={installAllVersions}
                >
                  Preparar todas
                </button>
              </div>
              <VersionGrid version={version()} onChoose={chooseVersion} large />
            </section>
          </Show>

          <Show when={panel() === "mods"}>
            <section class="rounded-md border border-line bg-surface p-6 shadow-soft">
              <div class="mb-5 flex items-center justify-between gap-3">
                <div>
                  <h3 class="text-2xl font-black">Mods y shaders</h3>
                  <p class="text-sm text-muted">Agrega archivos al perfil seleccionado.</p>
                </div>
                <span class="rounded-sm border border-blue/40 bg-blue/10 px-3 py-2 text-xs font-black uppercase text-blue2">
                  Por perfil
                </span>
              </div>
              <div class="flex gap-2 max-md:flex-col">
                <input
                  class="min-w-0 flex-1 rounded-md border border-line bg-panel px-3 py-4 text-sm text-ink outline-none placeholder:text-muted focus:border-blue2"
                  placeholder="Ruta del archivo"
                  value={importPath()}
                  onInput={(event) => setImportPath(event.currentTarget.value)}
                />
                <button
                  class="rounded-md bg-blue px-5 text-sm font-bold text-white transition hover:bg-blue2 hover:text-panel disabled:opacity-50"
                  disabled={busy() || !selectedProfile()}
                  onClick={() => importFile("mod")}
                >
                  Mod
                </button>
                <button
                  class="rounded-md bg-blue px-5 text-sm font-bold text-white transition hover:bg-blue2 hover:text-panel disabled:opacity-50"
                  disabled={busy() || !selectedProfile()}
                  onClick={() => importFile("shader")}
                >
                  Shader
                </button>
                <button
                  class="rounded-md bg-blue px-5 text-sm font-bold text-white transition hover:bg-blue2 hover:text-panel disabled:opacity-50"
                  disabled={busy() || !selectedProfile()}
                  onClick={() => importFile("modpack")}
                >
                  Modpack
                </button>
              </div>
            </section>
          </Show>
        </section>
      </div>
    </main>
  );
}

function VersionGrid(props: {
  version: string;
  onChoose: (id: string) => void;
  large?: boolean;
}) {
  return (
    <div
      class={`grid gap-3 ${
        props.large
          ? "grid-cols-4 max-2xl:grid-cols-3 max-lg:grid-cols-2 max-sm:grid-cols-1"
          : "grid-cols-4 max-2xl:grid-cols-3 max-md:grid-cols-2 max-sm:grid-cols-1"
      }`}
    >
      <For each={versionCards}>
        {(item) => (
          <button
            class={`group relative overflow-hidden rounded-md border px-4 py-4 text-left transition ${
              props.version === item.id
                ? "border-grass bg-grass/10 shadow-soft"
                : "border-line bg-panel hover:border-blue"
            } ${props.large ? "min-h-36" : "min-h-28"}`}
            onClick={() => props.onChoose(item.id)}
          >
            <div class="absolute inset-0 opacity-0 transition group-hover:opacity-100 bg-[radial-gradient(circle_at_20%_10%,rgba(56,189,248,0.18),transparent_35%)]" />
            <div class="relative z-10 flex h-full flex-col justify-between gap-5">
              <div class="flex items-start justify-between gap-3">
                <span class="text-2xl font-black">{item.id}</span>
                  <span class="rounded-sm border border-blue/40 bg-blue/10 px-2 py-1 text-[11px] font-bold uppercase text-blue2">
                  {item.note}
                </span>
              </div>
              <p class="text-sm font-semibold text-muted">{item.tag}</p>
            </div>
          </button>
        )}
      </For>
    </div>
  );
}

function StatusPanel(props: {
  selectedVersion: { id: string; tag: string; note: string };
  installReport: InstallVersionReport | null;
  launch: LaunchResult | null;
  modpack: ModpackImportReport | null;
}) {
  return (
    <aside class="rounded-md border border-line bg-surface p-5 shadow-soft">
      <h3 class="mb-4 text-lg font-black">Sesion</h3>
      <div class="space-y-3 text-sm">
        <div class="flex items-center justify-between border-b border-line pb-3">
          <span class="text-muted">Version</span>
          <span class="font-bold">{props.selectedVersion.id}</span>
        </div>
        <div class="flex items-center justify-between border-b border-line pb-3">
          <span class="text-muted">Tipo</span>
          <span class="font-bold">{props.selectedVersion.tag}</span>
        </div>
        <Show when={props.installReport}>
          {(report) => (
            <div class="rounded-md border border-blue/40 bg-blue/10 px-3 py-3">
              <p class="font-bold">Version preparada</p>
              <p class="mt-1 text-muted">
                {report().downloadedFiles} nuevos, {report().reusedFiles} listos
              </p>
            </div>
          )}
        </Show>
        <Show when={props.launch}>
          {(result) => (
            <div class="rounded-md border border-blue/40 bg-blue/10 px-3 py-3">
              <p class="font-bold">Juego iniciado</p>
              <p class="mt-1 text-muted">PID {result().pid}</p>
            </div>
          )}
        </Show>
        <Show when={props.modpack}>
          {(report) => (
            <div class="rounded-md border border-blue/40 bg-blue/10 px-3 py-3">
              <p class="font-bold">{report().name}</p>
              <p class="mt-1 text-muted">{report().filesDeclared} archivos</p>
            </div>
          )}
        </Show>
      </div>
    </aside>
  );
}
