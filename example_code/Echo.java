import java.util.Scanner;

public class Echo {
  public static void main(String[] args) {
    final Scanner scanner = new Scanner(System.in);
    if (scanner.hasNextLine()) {
      System.out.println(scanner.nextLine());
    }
  }
}
