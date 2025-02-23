<script>
    import { page } from "$app/state";
    import Navbar from "$lib/Navbar.svelte";
    import Post from "$lib/Post.svelte";
</script>

<div class="column">
    <Navbar />
    <span class="action align-self-center">Ваша лента</span>
    {#if page.data.feed === undefined || page.data.feed.length === 0 }
        <div id="feed-empty" class="column">
            <div class="text-center align-self-center">
                Подпишитесь на пользователей Хъ чтобы ваша лента имела
                активность
            </div>
            <div class="text-center align-self-center">
                Проверьте вкладку <a href="/latest">Последние посты</a> для того
                чтобы найти для себя новый контент
            </div>
        </div>
    {:else}
        <div id="posts-container" class="column feed-container">
            {#each page.data.feed as post}
                <Post {post} />
            {/each}
        </div>
    {/if}
</div>
