import '@birch/client/auto';

async function fetchTikTokVideos() {
  console.log('Fetching TikTok videos...');
  
  try {
    const response = await fetch('https://api.tiktok.com/v1/videos', {
      headers: {
        'Authorization': `Bearer ${process.env.TIKTOK_API_KEY}`,
        'Content-Type': 'application/json'
      }
    });

    if (!response.ok) {
      console.error(`Error: ${response.status} ${response.statusText}`);
      return;
    }

    const data = await response.json();
    console.log('Videos:', data);
  } catch (error) {
    console.error('Failed to fetch videos:', error);
  }
}

async function fetchOpenAICompletion(prompt: string) {
  console.log('Fetching OpenAI completion...');
  
  try {
    const response = await fetch('https://api.openai.com/v1/chat/completions', {
      method: 'POST',
      headers: {
        'Authorization': `Bearer ${process.env.OPENAI_API_KEY}`,
        'Content-Type': 'application/json'
      },
      body: JSON.stringify({
        model: 'gpt-4',
        messages: [{ role: 'user', content: prompt }]
      })
    });

    if (!response.ok) {
      console.error(`Error: ${response.status} ${response.statusText}`);
      return;
    }

    const data = await response.json();
    console.log('Completion:', data.choices[0].message.content);
  } catch (error) {
    console.error('Failed to fetch completion:', error);
  }
}

async function main() {
  console.log('Starting CLI script with Birch auto-rotation');
  console.log('---');
  
  await fetchTikTokVideos();
  console.log('---');
  
  await fetchOpenAICompletion('Hello, how are you?');
  console.log('---');
  
  console.log('Done!');
}

main();

