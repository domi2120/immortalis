# immortalis
 

## Getting Started
* run `docker compose up db pgadmin` to start postgres + pgadmin
* run `cd immortalis-backend-common && ~/.cargo/bin/diesel migration run` to run the migrations
* run `cd ../immortalis-client && pnpm i && pnpm run dev` to run the client (localhost:3000)
* run `cd ../immortalis-backend-api && cargo run` to run the api (localhost:8080, proxied through client at "localhost:3000/api")
* go to [pgadmin](http://localhost:5050/browser/) and execute the following code for some initial data
```
INSERT INTO public.videos(
	title, channel, views, upload_date, archived_date, duration, thumbnail_address, original_url)
	VALUES
	('Ghost - Rats (Official Music Video)', 'Ghost', 5000000, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 265, 'https://img.youtube.com/vi/C_ijc7A5oAc/maxresdefault.jpg', 'https://www.youtube.com/watch?v=C_ijc7A5oAc'),
	('I Am', 'Theocrary - Topic', 380000, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP, 660, 'https://img.youtube.com/vi/vfc8EjDuYNw/maxresdefault.jpg', 'https://www.youtube.com/watch?v=vfc8EjDuYNw');
	
insert into public.downloads (video_id, title, value)
select id, 'Download(1080p30)', 'Download(1080p30)'
from public.videos

insert into public.downloads (video_id, title, value)
select id, 'Audio Only', 'Audio Only'
from public.videos
```