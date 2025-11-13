import '@birch/client/auto';
import express from 'express';

const app = express();

app.get('/tweets', async (req, res) => {
  try {
    const response = await fetch('https://api.twitter.com/2/tweets/search/recent?query=nodejs', {
      headers: {
        'Authorization': `Bearer ${process.env.TWITTER_API_KEY}`
      }
    });

    if (!response.ok) {
      return res.status(response.status).json({ error: 'Twitter API error' });
    }

    const data = await response.json();
    res.json(data);
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
});

app.get('/tiktok', async (req, res) => {
  try {
    const response = await fetch('https://api.tiktok.com/v1/videos', {
      headers: {
        'Authorization': `Bearer ${process.env.TIKTOK_API_KEY}`
      }
    });

    if (!response.ok) {
      return res.status(response.status).json({ error: 'TikTok API error' });
    }

    const data = await response.json();
    res.json(data);
  } catch (error) {
    res.status(500).json({ error: 'Internal server error' });
  }
});

const PORT = process.env.PORT || 3000;

app.listen(PORT, () => {
  console.log(`Server running on port ${PORT}`);
  console.log('Birch auto-rotation is enabled');
});

