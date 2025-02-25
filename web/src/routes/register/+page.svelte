<script>
    import { goto, invalidate } from "$app/navigation";

    let realname;
    let username;
    let password;
    let passwordCheck;
</script>

<div class="column modal-auth">
    <!-- svelte-ignore a11y_missing_attribute -->
    <img src="/favicon.png" class="logo" />
    <span class="action">Зарегистрируйтесь в Хъ</span>
    <input
        type="text"
        class="text-box field margin-top-8"
        placeholder="Имя"
        bind:value={realname}
    />
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
    <input
        type="password"
        class="text-box field margin-top-8"
        placeholder="Повторите пароль"
        bind:value={passwordCheck}
    />
    <button
        class="button margin-top-8"
        onclick={async () => {
            if (password != passwordCheck) {
                alert("Пароли не совпадают");
                return;
            }
            if (password.length < 8) {
                alert("Пароль слишком короткий, нужно минимум 8 символов");
                return;
            }
            if (username.length < 3) {
                alert("Юзернейм слишком короткий, нужно минимум 3 символа");
                return;
            }
            if (realname.length === 0) {
                alert("Имя слишком короткое, нужен минимум 1 символ");
                return;
            }
            const response = await fetch("/api/auth/register", {
                method: "POST",
                headers: {
                    "Content-Type": "application/json",
                },
                body: JSON.stringify({
                    realname,
                    username,
                    password,
                }),
            });
            if (response.status == 200) {
                const data = await response.json();
                window.localStorage.setItem("token", data.token);
                goto("/", { invalidateAll: true });
            }
        }}>Зарегистрироваться</button
    >
    <a href="/login" class="margin-top-8">Уже зарегистрированы? Войдите</a>
</div>
