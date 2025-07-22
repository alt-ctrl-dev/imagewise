defmodule Imagewise.Application do
  # See https://hexdocs.pm/elixir/Application.html
  # for more information on OTP Applications
  @moduledoc false

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      ImagewiseWeb.Telemetry,
      {DNSCluster, query: Application.get_env(:imagewise, :dns_cluster_query) || :ignore},
      {Phoenix.PubSub, name: Imagewise.PubSub},
      # Start a worker by calling: Imagewise.Worker.start_link(arg)
      # {Imagewise.Worker, arg},
      # Start to serve requests, typically the last entry
      ImagewiseWeb.Endpoint
    ]

    # See https://hexdocs.pm/elixir/Supervisor.html
    # for other strategies and supported options
    opts = [strategy: :one_for_one, name: Imagewise.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    ImagewiseWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
