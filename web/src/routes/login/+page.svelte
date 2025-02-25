<script>
    import { goto, invalidate, invalidateAll } from "$app/navigation";

    let username;
    let password;
</script>

<div class="column modal-auth">
    <!-- svelte-ignore a11y_missing_attribute -->
    <img src="/favicon.png" class="logo" />
    <span class="action">Войдите в Хъ</span>
    <input
        type="text"
        class="text-box field margin-top-8"
        placeholder="Юзернейм"
        bind:value={username}
    />
    <input
        type="password"
        class="text-box field margin-top-8"
        placeholder="Пароль"
        bind:value={password}
    />
    <button
        class="button margin-top-8"
        onclick={async () => {
            const response = await fetch("/api/auth/login", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    username,
                    password,
                }),
            });
            if (response.status == 200) {
                const data = await response.json();
                window.localStorage.setItem("token", data.token);
                goto("/", { invalidateAll: true });
            } else {
                alert("Неверный логин или пароль");
            }
        }}>Войти</button
    >
    <a href="/register" class="margin-top-8">Нет аккаунта? Создайте новый</a>
</div>
