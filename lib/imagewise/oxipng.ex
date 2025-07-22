defmodule Imagewise.ImageProcessor.OxiPNG do
  use Rustler, otp_app: :imagewise, crate: "oxipng"

  # When your NIF is loaded, it will override this function.
  def add(_a, _b), do: :erlang.nif_error(:nif_not_loaded)

  # These functions are implemented in Rust.
  def resize_png(_png_data, _max_height), do: {:error, :nif_not_loaded}
  def minify_png(_png_data, _level), do: {:error, :nif_not_loaded}
  def png_to_webp(_png_data, _quality), do: {:error, :nif_not_loaded}

    @doc """
  Runs the full pipeline: SVG -> Resize -> Minify PNG -> WebP.
  """
  def full_conversion_pipeline(svg_path, opts \\ []) do
    resize_max_height = Keyword.get(opts, :resize_max_height, 32)
    minify_level = Keyword.get(opts, :minify_level, 2)
    webp_quality = Keyword.get(opts, :webp_quality, 75.0)

    # Use a temporary file for the initial SVG -> PNG step
    temp_path = Path.join(System.tmp_dir!(), "temp_#{System.unique_integer()}.png")

    with :ok <- Resvg.svg_to_png(svg_path, temp_path),
         {:ok, png_binary} <- File.read(temp_path) do
      # Clean up the temp file
      File.rm(temp_path)

      # Now perform the rest of the operations in memory, starting with resize
      with {:ok, resized_png_binary} <- Optimizer.resize_png(png_binary, resize_max_height),
           {:ok, minified_png_binary} <- Optimizer.minify_png(resized_png_binary, minify_level),
           {:ok, webp_binary} <- Optimizer.png_to_webp(resized_png_binary, webp_quality) do
        {:ok,
         %{
           minified_png: minified_png_binary,
           webp: webp_binary
         }}
      else
        {:error, reason} -> {:error, "Processing failed after SVG conversion: #{inspect(reason)}"}
      end
    else
      {:error, reason} ->
        {:error, "Failed during SVG conversion: #{inspect(reason)}"}
    end
  end
end
