export async function GET() {
  try {
    const response = await fetch('https://api.tiktok.com/v1/videos', {
      headers: {
        'Authorization': `Bearer ${process.env.TIKTOK_API_KEY}`
      }
    });

    if (!response.ok) {
      return Response.json(
        { error: 'TikTok API error' },
        { status: response.status }
      );
    }

    const data = await response.json();
    return Response.json(data);
  } catch (error) {
    return Response.json(
      { error: 'Internal server error' },
      { status: 500 }
    );
  }
}

