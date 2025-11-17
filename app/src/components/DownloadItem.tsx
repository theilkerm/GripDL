import { Pause, Play, X, CheckCircle2, AlertCircle } from "lucide-react";

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

interface DownloadItemProps {
  download: DownloadInfo;
  onPause: (id: string) => void;
  onResume: (id: string) => void;
  onCancel: (id: string) => void;
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const k = 1024;
  const sizes = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return `${parseFloat((bytes / Math.pow(k, i)).toFixed(2))} ${sizes[i]}`;
}

function getStatusIcon(status: DownloadInfo["status"]) {
  if (status === "Completed") {
    return <CheckCircle2 className="w-5 h-5 text-green-500" />;
  }
  if (typeof status === "object" && "Failed" in status) {
    return <AlertCircle className="w-5 h-5 text-red-500" />;
  }
  return null;
}

function getStatusText(status: DownloadInfo["status"]): string {
  if (typeof status === "object" && "Failed" in status) {
    return `Failed: ${status.Failed}`;
  }
  return status;
}

export default function DownloadItem({
  download,
  onPause,
  onResume,
  onCancel,
}: DownloadItemProps) {
  const progress =
    download.total_size && download.total_size > 0
      ? (download.downloaded_size / download.total_size) * 100
      : 0;

  const isActive = download.status === "Downloading" || download.status === "Pending";
  const isPaused = download.status === "Paused";
  const isCompleted = download.status === "Completed";
  const isFailed = typeof download.status === "object" && "Failed" in download.status;

  return (
    <div className="border rounded-lg p-4 bg-card">
      <div className="flex items-start justify-between mb-2">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 mb-1">
            {getStatusIcon(download.status)}
            <h3 className="font-medium text-foreground truncate">{download.file_name}</h3>
          </div>
          <p className="text-sm text-muted-foreground truncate">{download.url}</p>
        </div>
        <div className="flex items-center gap-2 ml-4">
          {isActive && (
            <button
              onClick={() => onPause(download.id)}
              className="p-2 hover:bg-muted rounded transition-colors"
              title="Pause"
            >
              <Pause className="w-4 h-4" />
            </button>
          )}
          {isPaused && (
            <button
              onClick={() => onResume(download.id)}
              className="p-2 hover:bg-muted rounded transition-colors"
              title="Resume"
            >
              <Play className="w-4 h-4" />
            </button>
          )}
          {!isCompleted && (
            <button
              onClick={() => onCancel(download.id)}
              className="p-2 hover:bg-muted rounded transition-colors text-destructive"
              title="Cancel"
            >
              <X className="w-4 h-4" />
            </button>
          )}
        </div>
      </div>

      <div className="mt-3">
        <div className="flex justify-between text-sm text-muted-foreground mb-1">
          <span>{getStatusText(download.status)}</span>
          <span>
            {formatBytes(download.downloaded_size)}
            {download.total_size && ` / ${formatBytes(download.total_size)}`}
          </span>
        </div>
        {isActive && download.total_size && (
          <div className="w-full bg-muted rounded-full h-2 overflow-hidden">
            <div
              className="bg-primary h-2 rounded-full transition-all duration-300"
              style={{ width: `${progress}%` }}
            />
          </div>
        )}
      </div>
    </div>
  );
}

