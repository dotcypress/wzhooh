<script>
  import 'bulma/css/bulma.min.css';
  import { onMount } from 'svelte';
  import {
    CornerDownLeftIcon,
    RefreshCwIcon,
    RepeatIcon,
    TerminalIcon,
    Trash2Icon
  } from 'svelte-feather-icons';

  /**
   * @type any
   */
  let port;
  let error = '';
  let busy = false;
  let connected = false;
  let navbarOpen = false;

  let laps = "0";

  let decoder = new TextDecoder('utf-8');

  onMount(async () => {});

  async function connect() {
    const usbVendorId = 0x16c0;
    // @ts-ignore
    port = await navigator.serial.requestPort({ filters: [{ usbVendorId }] });
    await port.open({ baudRate: 115200 });
    // @ts-ignore
    window.port = port;
    read_port();
  }

  async function read_port() {
    while (port && port.readable) {
      const reader = port.readable.getReader();
      try {
        while (true) {
          const { value, done } = await reader.read();
          let line = decoder.decode(value);
          const regex = /Track #(\d): (\d+)?/gm;
          let match = regex.exec(line);
          if (match != null) {
            laps = match[2];
          }
          if (done) {
            break;
          }
        }
      } catch (error) {
        console.log(error);
      } finally {
        reader.releaseLock();
      }
    }
    setTimeout(read_port, 100);
  }

  function toggleConnection() {
    connect();
  }

  function toggleNavbar() {
    navbarOpen = !navbarOpen;
  }

  function reload() {}
</script>

<div class="notification-container">
  {#if error}
    <div class="notification is-danger">{error}</div>
  {/if}
</div>

<div class="workspace">
  <nav class="navbar is-dark">
    <div class="navbar-brand">
      <div class="navbar-item">
        <span
          class="tag is-info is-family-code is-size-6 has-text-weight-bold"
          class:is-danger={connected}
        >
          <span class="icon is-small">
            <TerminalIcon />
          </span>
          &nbsp; web_serial
        </span>
      </div>
      <button
        class="navbar-burger burger has-background-dark"
        aria-hidden="true"
        class:is-active={navbarOpen}
        on:click={toggleNavbar}
      >
        <span aria-hidden="true" />
        <span aria-hidden="true" />
        <span aria-hidden="true" />
      </button>
    </div>
    <div class="navbar-menu has-background-dark" class:is-active={navbarOpen}>
      <div class="navbar-start">
        <div class="navbar-item">
          <div class="buttons are-small">
            <!-- <button
              class="button is-small is-danger is-outlined"
              title="Clear"
              on:click={clear}
            >
              <span class="icon is-small">
                <Trash2Icon />
              </span>
            </button> -->
          </div>
        </div>
      </div>
      <div class="navbar-end">
        <div class="navbar-item">
          <div class="buttons are-small">
            <!-- <button
              class="button is-small is-primary is-outlined"
              title="Auto EOL"
              class:is-light={convertEol}
              on:click={toggleEol}
            >
              <span class="icon is-small">
                <CornerDownLeftIcon />
              </span>
            </button>
            <button
              class="button is-small is-info is-outlined"
              title="Local echo"
              class:is-light={localEcho}
              on:click={toggleLocalEcho}
            >
              <span class="icon is-small">
                <RepeatIcon />
              </span>
            </button> -->
          </div>
        </div>
        <div class="navbar-item">
          <div class="field has-addons">
            <!-- <p class="control">
              <button
                class="button is-small"
                title="Reload"
                disabled={connected}
                on:click={reloadPorts}
              >
                <span class="icon is-small">
                  <RefreshCwIcon />
                </span>
              </button>
            </p> -->
            <!-- <div class="control">
              <input
                class="input is-small port"
                list="ports"
                bind:value={portName}
                disabled={busy || connected}
              />
              <datalist id="ports">
                {#each ports as port}
                  <option value={port} />{/each}
              </datalist>
            </div>
            <p class="control">
              <input
                class="input is-small baudrate"
                type="text"
                placeholder="Baudrate"
                bind:value={baudrate}
                disabled={busy || connected}
              />
            </p> -->
            <p class="control">
              <button
                class="button is-small"
                class:is-loading={busy}
                class:is-success={!connected}
                class:is-danger={connected}
                disabled={busy}
                on:click={toggleConnection}
              >
                {connected ? 'Disconnect' : 'Connect'}
              </button>
            </p>
          </div>
        </div>
      </div>
    </div>
  </nav>

  <section class="hero is-link">
    <div class="hero-body">
      <p class="title">
        Track #1 laps
      </p>
      <p class="title">
        {laps}
      </p>
    </div>
  </section>
</div>
<style>
  :global(html),
  :global(body) {
    padding: 0;
    margin: 0;
    min-height: 100%;
  }
  .notification-container {
    position: absolute;
    z-index: 1;
    bottom: 0;
    right: 0;
    padding: 1em;
  }
  .workspace {
    display: flex;
    flex-direction: column;
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
  }
  .workspace .navbar button,
  .workspace .navbar button:focus {
    outline: none;
    border: none;
  }
</style>
