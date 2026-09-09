import {
  File,
  Image,
  FileText,
  Calculator,
  Zap,
  Ruler,
  Clock,
  AppWindow,
  HardDrive,
} from "lucide-react";
import { SearchResult } from "../types";

interface PreviewPanelProps {
  result: SearchResult;
  query: string;
}

export default function PreviewPanel({ result, query }: PreviewPanelProps) {
  const renderPreview = () => {
    switch (result.category) {
      case "Application":
        return <ApplicationPreview result={result} />;
      case "File":
        return <FilePreview result={result} />;
      case "Calculator":
        return <CalculatorPreview result={result} />;
      case "Action":
        return <ActionPreview result={result} />;
      case "Conversion":
        return <ConversionPreview result={result} />;
      case "Timezone":
        return <TimezonePreview result={result} />;
      default:
        return <GenericPreview result={result} />;
    }
  };

  return (
    <div className="preview-panel">
      <div className="mb-4">
        <h3 className="text-lg font-semibold text-text-primary truncate">
          {result.name}
        </h3>
        <p className="text-sm text-text-secondary mt-1">{result.category}</p>
      </div>
      {renderPreview()}
      <div className="mt-4 pt-4 border-t border-white/10">
        <div className="text-xs text-text-muted">
          <div className="flex justify-between">
            <span>Path:</span>
            <span className="text-text-secondary truncate ml-2 max-w-[180px]">
              {result.path}
            </span>
          </div>
          {result.metadata && (
            <div className="flex justify-between mt-1">
              <span>Info:</span>
              <span className="text-text-secondary truncate ml-2 max-w-[180px]">
                {result.metadata}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

function ApplicationPreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-accent-blue/20 flex items-center justify-center">
          <AppWindow className="w-6 h-6 text-accent-blue" />
        </div>
        <div>
          <div className="font-medium">{result.name}</div>
          <div className="text-xs text-text-secondary">Application</div>
        </div>
      </div>
      <div className="text-sm text-text-secondary">
        Press Enter to launch this application
      </div>
    </div>
  );
}

function FilePreview({ result }: { result: SearchResult }) {
  const ext = result.icon || "";
  const isImage = ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg"].includes(ext.toLowerCase());
  const isText = ["txt", "md", "json", "xml", "csv", "log", "ini", "cfg"].includes(ext.toLowerCase());
  const isPdf = ext.toLowerCase() === "pdf";

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-white/10 flex items-center justify-center">
          {isImage ? (
            <Image className="w-6 h-6 text-green-400" />
          ) : isPdf ? (
            <FileText className="w-6 h-6 text-red-400" />
          ) : (
            <File className="w-6 h-6 text-blue-400" />
          )}
        </div>
        <div>
          <div className="font-medium">{result.name}</div>
          <div className="text-xs text-text-secondary">
            {ext.toUpperCase()} File • {result.path.split("\\").slice(0, -1).join("\\")}
          </div>
        </div>
      </div>
      {isImage && (
        <div className="text-sm text-text-secondary">
          Press Space to preview this image
        </div>
      )}
      {isText && (
        <div className="text-sm text-text-secondary">
          Press Enter to open in default editor
        </div>
      )}
      <div className="flex gap-2 text-xs">
        <span className="px-2 py-1 rounded bg-white/5 text-text-muted">
          Ctrl+Enter: Open folder
        </span>
        <span className="px-2 py-1 rounded bg-white/5 text-text-muted">
          Ctrl: Show path
        </span>
      </div>
    </div>
  );
}

function CalculatorPreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-accent-blue/20 flex items-center justify-center">
          <Calculator className="w-6 h-6 text-accent-blue" />
        </div>
        <div>
          <div className="font-medium text-2xl">{result.name}</div>
          <div className="text-xs text-text-secondary">Result</div>
        </div>
      </div>
      <div className="text-sm text-text-secondary">
        Press Enter to copy result to clipboard
      </div>
    </div>
  );
}

function ActionPreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-orange-500/20 flex items-center justify-center">
          <Zap className="w-6 h-6 text-orange-500" />
        </div>
        <div>
          <div className="font-medium">{result.name}</div>
          <div className="text-xs text-text-secondary">{result.path}</div>
        </div>
      </div>
      <div className="text-sm text-text-secondary">
        Press Enter to execute this action
      </div>
    </div>
  );
}

function ConversionPreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-green-500/20 flex items-center justify-center">
          <Ruler className="w-6 h-6 text-green-500" />
        </div>
        <div>
          <div className="font-medium text-xl">{result.name}</div>
          <div className="text-xs text-text-secondary">Conversion Result</div>
        </div>
      </div>
      <div className="text-sm text-text-secondary">
        Press Enter to copy result to clipboard
      </div>
    </div>
  );
}

function TimezonePreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-purple-500/20 flex items-center justify-center">
          <Clock className="w-6 h-6 text-purple-500" />
        </div>
        <div>
          <div className="font-medium text-xl">{result.name}</div>
          <div className="text-xs text-text-secondary">Current Time</div>
        </div>
      </div>
      <div className="text-sm text-text-secondary">
        Press Enter to copy time to clipboard
      </div>
    </div>
  );
}

function GenericPreview({ result }: { result: SearchResult }) {
  return (
    <div className="space-y-3">
      <div className="flex items-center gap-3">
        <div className="w-12 h-12 rounded-lg bg-white/10 flex items-center justify-center">
          <HardDrive className="w-6 h-6 text-text-secondary" />
        </div>
        <div>
          <div className="font-medium">{result.name}</div>
          <div className="text-xs text-text-secondary">{result.category}</div>
        </div>
      </div>
    </div>
  );
}
