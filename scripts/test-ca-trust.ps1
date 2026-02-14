$pipe = New-Object System.IO.Pipes.NamedPipeClientStream(".", "localdomain", [System.IO.Pipes.PipeDirection]::InOut)
try {
    $pipe.Connect(3000)
    $writer = New-Object System.IO.StreamWriter($pipe)
    $reader = New-Object System.IO.StreamReader($pipe)
    $msg = '{"jsonrpc":"2.0","method":"install_ca_trust","params":null,"id":1}'
    $writer.WriteLine($msg)
    $writer.Flush()
    $response = $reader.ReadLine()
    Write-Host "install_ca_trust response: $response"
    $pipe.Close()
} catch {
    Write-Host "Error: $_"
}
