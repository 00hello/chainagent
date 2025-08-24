// Defining a data model.
class Resume {
  name string
  email string
  experience string[]
  skills string[]
}

class NativeBalance {
  who string
}

class FungibleBalance {
  token string
  who string
}

class Code {
  addr string
}


// Create a function to extract the resume from a string.
function ExtractResume(resume: string) -> Resume {
  // Specify a client as provider/model-name
  // you can use custom LLM params with a custom client name from clients.baml like "client CustomHaiku"
  client "openai/gpt-4o" // Set OPENAI_API_KEY to use this client.
  prompt #"
    Extract from this content:
    {{ resume }}

    {{ ctx.output_format }}
  "#
}

function GetNativeBalance(who: string) -> string {
  client "openai/gpt-4o"
  prompt #"
    Get the native balance of the given address.
    {{ who }}
  "#
}

function GetFungibleBalance(token: string, who: string) -> string {
  client "openai/gpt-4o"
  prompt #"
    Get the fungible balance of the given token and address.
    {{ token }}
    {{ who }}
  "#
}

function GetCode(addr: string) -> string {
  client "openai/gpt-4o"
  prompt #"
    Get the code of the given address.
    {{ addr }}
  "#
}

// Test the function with a sample resume. Open the VSCode playground to run this.
test vaibhav_resume {
  functions [ExtractResume, GetNativeBalance, GetFungibleBalance, GetCode]
  args {
    resume #"
      Vaibhav Gupta
      vbv@boundaryml.com

      Experience:
      - Founder at BoundaryML
      - CV Engineer at Google
      - CV Engineer at Microsoft

      Skills:
      - Rust
      - C++
    "#
  }

}
