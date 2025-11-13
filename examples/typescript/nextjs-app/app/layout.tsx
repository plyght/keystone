import '@birch/client/auto';
import type { Metadata } from 'next';

export const metadata: Metadata = {
  title: 'Birch Auto-Rotation Example',
  description: 'Next.js app with automatic API key rotation',
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en">
      <body>{children}</body>
    </html>
  );
}

