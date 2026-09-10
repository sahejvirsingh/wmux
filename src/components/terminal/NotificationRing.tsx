import "./NotificationRing.css";

export function NotificationRing({ attention }: { attention: boolean }) {
  if (!attention) {
    return null;
  }
  return <div className="notification-ring" aria-label="unread notifications" />;
}
