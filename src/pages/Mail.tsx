import { useState } from "react";
import { Mail as MailIcon, Trash2, Eye } from "lucide-react";
import { useMailStore, type Email } from "../stores/mail";

function EmailRow({
  email,
  isSelected,
  onSelect,
}: {
  email: Email;
  isSelected: boolean;
  onSelect: () => void;
}) {
  return (
    <button
      onClick={onSelect}
      className={`w-full text-left px-4 py-3 border-b border-border transition-colors ${
        isSelected ? "bg-primary-light" : "hover:bg-surface-secondary"
      } ${!email.read ? "font-medium" : ""}`}
    >
      <div className="flex items-center justify-between mb-1">
        <span className="text-sm text-text-primary truncate max-w-[200px]">
          {email.from}
        </span>
        <span className="text-xs text-text-muted">{email.timestamp}</span>
      </div>
      <p className="text-sm text-text-primary truncate">{email.subject}</p>
      {email.appName && (
        <span className="text-xs text-primary">{email.appName}</span>
      )}
    </button>
  );
}

export default function Mail() {
  const emails = useMailStore((s) => s.emails);
  const clearAll = useMailStore((s) => s.clearAll);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const selectedEmail = emails.find((e) => e.id === selectedId);

  return (
    <div>
      <div className="flex items-center justify-between mb-8">
        <div>
          <h1 className="text-2xl font-bold text-text-primary">Mail</h1>
          <p className="text-text-secondary mt-1">
            Captured emails from your applications (SMTP port 2525)
          </p>
        </div>
        {emails.length > 0 && (
          <button
            onClick={clearAll}
            className="flex items-center gap-2 px-4 py-2 rounded-lg border border-border text-sm font-medium text-danger hover:bg-danger/5 transition-colors"
          >
            <Trash2 className="w-4 h-4" />
            Clear All
          </button>
        )}
      </div>

      {emails.length > 0 ? (
        <div className="flex bg-surface rounded-xl border border-border overflow-hidden h-[600px]">
          {/* Email list */}
          <div className="w-80 border-r border-border overflow-y-auto shrink-0">
            {emails.map((email) => (
              <EmailRow
                key={email.id}
                email={email}
                isSelected={selectedId === email.id}
                onSelect={() => setSelectedId(email.id)}
              />
            ))}
          </div>

          {/* Email preview */}
          <div className="flex-1 overflow-y-auto">
            {selectedEmail ? (
              <div className="p-6">
                <h2 className="text-xl font-bold text-text-primary mb-2">
                  {selectedEmail.subject}
                </h2>
                <div className="flex items-center gap-4 mb-4 text-sm text-text-secondary">
                  <span>From: {selectedEmail.from}</span>
                  <span>To: {selectedEmail.to.join(", ")}</span>
                </div>
                <div className="border-t border-border pt-4">
                  {selectedEmail.htmlBody ? (
                    <div
                      dangerouslySetInnerHTML={{
                        __html: selectedEmail.htmlBody,
                      }}
                    />
                  ) : (
                    <pre className="text-sm text-text-primary whitespace-pre-wrap">
                      {selectedEmail.textBody}
                    </pre>
                  )}
                </div>
              </div>
            ) : (
              <div className="flex items-center justify-center h-full">
                <div className="text-center">
                  <Eye className="w-8 h-8 text-text-muted mx-auto mb-2" />
                  <p className="text-text-secondary">
                    Select an email to preview
                  </p>
                </div>
              </div>
            )}
          </div>
        </div>
      ) : (
        <div className="text-center py-16">
          <MailIcon className="w-12 h-12 text-text-muted mx-auto mb-4" />
          <h3 className="text-lg font-medium text-text-primary mb-2">
            No emails captured
          </h3>
          <p className="text-text-secondary">
            Configure your app's MAIL_PORT to 2525 to capture emails here
          </p>
        </div>
      )}
    </div>
  );
}
