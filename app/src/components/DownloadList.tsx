import DownloadItem from "./DownloadItem";

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

interface DownloadListProps {
  downloads: DownloadInfo[];
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
}

export default function DownloadList({
  downloads,
  onPause,
  onResume,
  onCancel,
}: DownloadListProps) {
  if (downloads.length === 0) {
    return (
      <div className="text-center py-12 text-muted-foreground">
        <p>No downloads yet</p>
        <p className="text-sm mt-2">Downloads from Firefox will appear here</p>
      </div>
    );
  }

  return (
    <div className="space-y-4">
      {downloads.map((download) => (
        <DownloadItem
          key={download.id}
          download={download}
          onPause={onPause}
          onResume={onResume}
          onCancel={onCancel}
        />
      ))}
    </div>
  );
}

