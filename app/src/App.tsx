import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import DownloadList from "./components/DownloadList";
import DownloadItem from "./components/DownloadItem";

interface DownloadInfo {
  id: string;
  url: string;
  file_path: string;
  file_name: string;
  total_size: number | null;
  downloaded_size: number;
  status: "Pending" | "Downloading" | "Paused" | "Completed" | { Failed: string } | "Cancelled";
  cookies: string | null;
  referrer: string | null;
  user_agent: string | null;
  created_at: number;
  updated_at: number;
}

function App() {
  const [downloads, setDownloads] = useState<DownloadInfo[]>([]);

  useEffect(() => {
    // Load initial downloads
    loadDownloads();

    // Listen for download updates
    const unlisten = listen<DownloadInfo>("download-update", (event) => {
      setDownloads((prev) => {
        const index = prev.findIndex((d) => d.id === event.payload.id);
        if (index >= 0) {
          const updated = [...prev];
          updated[index] = event.payload;
          return updated;
        }
        return [...prev, event.payload];
      });
    });

    // Listen for native download requests from extension
    const unlistenNative = listen<any>("native-download-request", async (event) => {
      const { url, cookies, referrer, user_agent } = event.payload;
      await startDownload(url, cookies, referrer, user_agent);
    });

    return () => {
      unlisten.then((fn) => fn());
      unlistenNative.then((fn) => fn());
    };
  }, []);

  const loadDownloads = async () => {
    try {
      const result = await invoke<DownloadInfo[]>("get_downloads");
      setDownloads(result);
    } catch (error) {
      console.error("Failed to load downloads:", error);
    }
  };

  const startDownload = async (
    url: string,
    cookies?: string,
    referrer?: string,
    userAgent?: string
  ) => {
    try {
      await invoke("start_download", {
        url,
        cookies: cookies || null,
        referrer: referrer || null,
        userAgent: userAgent || null,
      });
      await loadDownloads();
    } catch (error) {
      console.error("Failed to start download:", error);
    }
  };

  const pauseDownload = async (id: string) => {
    try {
      await invoke("pause_download", { id });
      await loadDownloads();
    } catch (error) {
      console.error("Failed to pause download:", error);
    }
  };

  const resumeDownload = async (id: string) => {
    try {
      await invoke("resume_download", { id });
      await loadDownloads();
    } catch (error) {
      console.error("Failed to resume download:", error);
    }
  };

  const cancelDownload = async (id: string) => {
    try {
      await invoke("cancel_download", { id });
      await loadDownloads();
    } catch (error) {
      console.error("Failed to cancel download:", error);
    }
  };

  return (
    <div className="min-h-screen bg-background">
      <div className="container mx-auto p-6">
        <header className="mb-8">
          <h1 className="text-3xl font-bold text-foreground">GripDL</h1>
          <p className="text-muted-foreground mt-2">macOS Download Manager</p>
        </header>

        <DownloadList
          downloads={downloads}
          onPause={pauseDownload}
          onResume={resumeDownload}
          onCancel={cancelDownload}
        />
      </div>
    </div>
  );
}

export default App;

