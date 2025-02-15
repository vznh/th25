mod suggestion;

fn main() {
    let example_xml = r#"
    <Class>
      <ClassName>SwapUtil</ClassName>
      <ReviewInfo>
        <CommitId>f7e1a4b</CommitId>
        <FilePath>src/SwapUtil.py</FilePath>
      </ReviewInfo>
      <Methods>
        <!-- Method 1: swap -->
        <Method>
          <Name>swap</Name>
          <Description>Swaps two variables.</Description>
          <Parameters>
            <Parameter>
              <Name>a</Name>
              <Type>int</Type>
            </Parameter>
            <Parameter>
              <Name>b</Name>
              <Type>int</Type>
            </Parameter>
          </Parameters>
          <Body>
            <Statement>temp = a</Statement>
            <Statement>a = b</Statement>
            <Statement>b = temp</Statement>
            <Statement>return a, b</Statement>
          </Body>
        </Method>

        <!-- Method 2: handle_swap -->
        <Method>
          <Name>handle_swap</Name>
          <Description>Handles the swap by calling another method.</Description>
          <Parameters>
            <Parameter>
              <Name>value1</Name>
              <Type>int</Type>
            </Parameter>
            <Parameter>
              <Name>value2</Name>
              <Type>int</Type>
            </Parameter>
          </Parameters>
          <Body>
            <Statement>self.perform_swap(value1, value2)</Statement>
          </Body>
        </Method>

        <!-- Method 3: perform_swap -->
        <Method>
          <Name>perform_swap</Name>
          <Description>Performs the swap by calling another method.</Description>
          <Parameters>
            <Parameter>
              <Name>x</Name>
              <Type>int</Type>
            </Parameter>
            <Parameter>
              <Name>y</Name>
              <Type>int</Type>
            </Parameter>
          </Parameters>
          <Body>
            <Statement>x, y = self.swap(x, y)</Statement>
            <Statement>return x, y</Statement>
          </Body>
        </Method>
      </Methods>
    </Class>
    "#;
    let result = suggestion::send_request_to_groq(example_xml);
    println!("Response from Groq: {}", result);
}
